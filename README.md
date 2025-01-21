Rust Screen Sharing
======================

The app offers efficient tools for **screen sharing**, allowing users to view shared content in real time. It is ideal for presentations, technical support, or remote work sessions.

**Key Features:**

1. **Real-time Screen Sharing:**

   * Enables users to share their device's screen seamlessly and without interruptions.
   * Supports multiple screens and partial screen selection.
   * Compatible with various devices and platforms (Windows, MacOS, and Linux).
   * The application can also be used to receive video streams, offering a comprehensive solution for both screen sharing and viewing.

2. **Session Recording:**

   * Saves screen-sharing sessions as videos for future use.
   * Videos are saved in FLV format (VLC Media Player is recommended).

3. **Hotkey Support:**

   * Provides keyboard shortcuts for functionalities like stopping, darkening, and pausing the stream.

4. **Intuitive User Interface:**

   * Minimalistic and easy-to-navigate design suitable for all user types.

## Installing

### Windows

To run the application, the installation of the Gstreamer framework is required:

* [Guide Link](https://gstreamer.freedesktop.org/documentation/installing/on-windows.html?gi-language=c)
* [Binaries Download Link](https://gstreamer.freedesktop.org/download/#windows)

The application uses additional plugins beyond the basic ones for capture, encoding, etc., which are often not available for separate installation. Therefore, a complete installation of the software is highly recommended.

### Linux

Tested on Ubuntu 24.04  
To run the application, you need to install some packages using the following commands:

sudo apt-get install libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev libgstreamer-plugins-bad1.0-dev gstreamer1.0-plugins-base gstreamer1.0-plugins-good gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly gstreamer1.0-libav gstreamer1.0-tools gstreamer1.0-x gstreamer1.0-alsa gstreamer1.0-gl gstreamer1.0-gtk3 gstreamer1.0-qt5 gstreamer1.0-pulseaudio
sudo apt-get install build-essential


### MacOS

(to do)

## HotKeys:

CTRL + P: pause streaming
CTRL + R: restart streaming
CTRL + S: start streaming


