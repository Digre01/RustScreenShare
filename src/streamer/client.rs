use gst::prelude::*;
use rfd::FileDialog;
use gst::{ClockTime, Element, Pipeline, State};
use std::{thread};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use crate::streamer::error::ClientError;
#[cfg(target_os = "macos")]
use objc::{class, msg_send, sel, sel_impl};



#[cfg(target_os = "macos")]
fn initialize_macos_app() {
    unsafe {
        //macOS initialization
        let _: () = msg_send![class!(NSApplication), sharedApplication];

    }
}


pub struct StreamerClient {
    pipeline: Option<Pipeline>,
    is_streaming: Arc<Mutex<bool>>,
    is_recording: Arc<Mutex<bool>>,
    tee: Option<Element> //Allows streaming and recording to happen simultaneously in 2 different pipelines
}

impl StreamerClient {
    pub fn new(ip: String, port: i32) -> Result<Self, ClientError> {
        gst::init().unwrap();

       
        #[cfg(target_os = "macos")]
        {
            initialize_macos_app();
        }

        println!("IP:{} Port: {}", ip,port);

        let pipeline = Pipeline::new();

        let udpsrc = gst::ElementFactory::make("udpsrc")
            .property("port", &port)
            .property("address", &ip)
            .property("caps", &gst::Caps::new_empty_simple("application/x-rtp"))
            .build()
            .map_err(|_| ClientError { message: "Failed to create element 'udpsrc'".to_string() })?;


        let queue = gst::ElementFactory::make("queue").build()
            .map_err(|_| ClientError {
                message: "Failed to create queue".to_string(),
            })?;
        let rtph264depay = gst::ElementFactory::make("rtph264depay")
            .build()
            .map_err(|_| ClientError { message: "Failed to create element 'rtph264depay'".to_string() })?;

        let tee = gst::ElementFactory::make("tee")
            .name("tee")
            .build()
            .map_err(|_| ClientError { message: "Failed to create element 'tee'".to_string() })?;

        let queue_display = gst::ElementFactory::make("queue").build()
            .map_err(|_| ClientError {
                message: "Failed to create queue_display".to_string(),
            })?;


        let avdec_h264 = gst::ElementFactory::make("avdec_h264")

            .build()
            .map_err(|_| ClientError { message: "Failed to create element 'avdec_h264'".to_string() })?;

        let videoconvert = gst::ElementFactory::make("videoconvert")
            .build()
            .map_err(|_| ClientError { message: "Failed to create element 'videoconvert'".to_string() })?;

        let autovideosink = gst::ElementFactory::make("autovideosink")
            .property("sync", true)  
            .build()
            .map_err(|_| ClientError { message: "Failed to create element 'autovideosink'".to_string() })?;

        pipeline.add_many(&[
            &udpsrc,
            &queue,
            &rtph264depay,
            &tee,
            &queue_display,
            &avdec_h264,
            &videoconvert,
            &autovideosink,
        ]).map_err(|_| ClientError { message: "Failed to add elements to pipeline".to_string() })?;

        gst::Element::link_many(&[
            &udpsrc,
            &queue,
            &rtph264depay,
            &tee,
            &queue_display,
            &avdec_h264,
            &videoconvert,
            &autovideosink,
        ]).map_err(|_| ClientError { message: "Failed to link elements".to_string() })?;

        pipeline.set_state(State::Ready).expect("Unable to set the pipeline to the `Ready` state");

        Ok(Self {
            pipeline: Some(pipeline),
            is_streaming: Arc::new(Mutex::new(false)),
            is_recording: Arc::new(Mutex::new(false)),
            tee: Some(tee),
        })
    }


    pub fn get_is_rec(&self)-> bool{

        let is_recording = self.is_recording.lock().unwrap();
        if *is_recording {
            return true;
        }
        return  false;

        
    }

    pub fn start_streaming(&mut self) -> Result<(), ClientError> {
        if let Some(ref pipeline) = self.pipeline {
            pipeline.set_state(State::Playing)
                .map_err(|_| ClientError { message: "Failed to start playing".to_string() })?;

            let bus = pipeline.bus().unwrap();
            let is_streaming = Arc::clone(&self.is_streaming);
            let pipeline_clone = self.pipeline.clone();

            thread::spawn(move || {
                let timeout = Duration::from_secs(30000);
                let mut last_msg_time = std::time::Instant::now();

                loop {
                    match bus.timed_pop(ClockTime::from_seconds(timeout.as_secs())) {
                        Some(msg) => {
                            last_msg_time = std::time::Instant::now();
                            match msg.view() {
                                gst::MessageView::Eos(..) => {
                                    println!("End of stream");
                                    let mut streaming = is_streaming.lock().unwrap();
                                    *streaming = false;
                                    break;
                                }
                                gst::MessageView::Error(err) => {
                                    
                                    let mut streaming = is_streaming.lock().unwrap();
                                    *streaming = false;

                                    return Err(ClientError {
                                        message: format!("Error from element {:?}: {}",
                                                         err.src().map(|s| s.path_string()),
                                                         err.error()),
                                    });
                                }
                                _ => (),
                            }
                        }
                        None => {
                            if last_msg_time.elapsed() >= timeout {
                                println!("No messages received for a while. Stream is ending.");
                                let mut streaming = is_streaming.lock().unwrap();
                                *streaming = false;
                                break;
                            }
                        }
                    }
                }
                if let Some(pipeline) = pipeline_clone {
                    pipeline.set_state(State::Null).unwrap();
                }
                println!("Closing render window...");
                Ok(())

            });

            let mut streaming = self.is_streaming.lock().unwrap();
            *streaming = true;
        }
        Ok(())
    }

    pub fn stop_streaming(&mut self) {
        if let Some(ref pipeline) = self.pipeline {
            pipeline.set_state(State::Null).unwrap();
        }
        self.pipeline = None;
        println!("Streaming stopped.");
    }



    pub fn start_recording(&mut self) -> Result<(), ClientError> {
        let mut is_recording = self.is_recording.lock().unwrap();
        if *is_recording {
            return Err(ClientError {
                message: "Recording is already in progress.".to_string(),
            });
        }




        let file_path = FileDialog::new()
        .set_title("Choose where to save the video")
        .set_file_name("output_video.mp4")
        .add_filter("FLV Video", &["flv"]) // Allow only `.flv` extensions
        .save_file();

         let path_str = match file_path {
                Some(path) => path.to_str().ok_or(ClientError {
                    message: "Invalid file path".to_string()})?
                    .to_string(), 
                None => "output_video.flv".to_string(), // Default file name
            };


        if let Some(ref pipeline) = self.pipeline {
            if let Some(ref _tee) = self.tee {

                // Create new elements for recording
                let queue_record = gst::ElementFactory::make("queue").build().map_err(|_| ClientError {
                    message: "Failed to create queue for recording".to_string(),
                })?;

                let h264parse = gst::ElementFactory::make("h264parse")
                    .build()
                    .map_err(|_| ClientError {
                        message: "Failed to create h264parse".to_string(),
                    })?;

                let flvmux = gst::ElementFactory::make("flvmux").build().map_err(|_| ClientError {
                    message: "Failed to create flvmux".to_string(),
                })?;
                let filesink = gst::ElementFactory::make("filesink")
                    .property("location", path_str)
                    .build()
                    .map_err(|_| ClientError {
                        message: "Failed to create filesink".to_string(),
                    })?;



                // Add new elements to the pipeline
                pipeline.add_many(&[&queue_record, &h264parse, &flvmux, &filesink])
                    .map_err(|_| ClientError { message: "Failed to add recording elements to pipeline".to_string() })?;

                // Link the elements for the recording branch
                gst::Element::link_many(&[&queue_record, &h264parse, &flvmux, &filesink])
                    .map_err(|_| ClientError { message: "Failed to link recording elements".to_string() })?;


                queue_record.sync_state_with_parent().unwrap();
                h264parse.sync_state_with_parent().unwrap();
                flvmux.sync_state_with_parent().unwrap();
                filesink.sync_state_with_parent().unwrap();

                let tee = pipeline.by_name("tee").unwrap();


                let tee_pad_templ = tee.pad_template("src_%u").unwrap();
                let tee_pad = tee.request_pad(&tee_pad_templ, None, None).unwrap();
                let sink_pad = queue_record.static_pad("sink").unwrap();
                match tee_pad.link(&sink_pad) {
                    Ok(_) => {
                        println!("Recording branch linked.");
                    }
                    Err(e) => {
                        panic!("Failed to link tee and queue_record, {:?}", e);
                    }
                }


                *is_recording = true;
                println!("Recording started.");
            }
        }

        Ok(())
    }

    pub fn stop_recording(&mut self) -> Result<(), ClientError> {  
        let mut is_recording = self.is_recording.lock().unwrap();
        if !*is_recording {
            return Err(ClientError {
                message: "No recording in progress.".to_string(),
            });
        }
        println!("Stop recording");
        *is_recording = false;


        Ok(())
    }
}
