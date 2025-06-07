# RUST BINDINGS FOR ZOOM SDK LINUX

This project provides **Rust bindings** for the **Zoom SDK on Linux**, specifically tailored for integration with Zoom meeting functionalities.

Below are instructions to set up the environment and get everything working smoothly on a Debian-based system.

---

## Installation

### Step 1: Install Required Dependencies

Before starting, make sure you have the necessary libraries installed:

```bash
apt install libxcb libglib2.0-0 libglib2.0-dev patchelf
```

### Step 2: Obtain the Zoom Linux SDK

Download the Linux version of the Zoom SDK from the official [Zoom Marketplace](https://zoom.us) and place the files in the `zoom-meeting-sdk-linux` directory of your project.

### Step 3: Create a Symbolic Link for the Main Library

Inside the `zoom-meeting-sdk-linux` directory, create a symbolic link to the main library file `libmeetingsdk.so`:

```bash
cd zoom-meeting-sdk-linux && ln -s libmeetingsdk.so libmeetingsdk.so.1
```

### Step 4: Add Missing Dependency to the Library

The Zoom SDK library may have a missing dependency. Add it manually using `patchelf`:

```bash
patchelf --add-needed /usr/lib/x86_64-linux-gnu/libgio-2.0.so libmeetingsdk.so
```

---

## Launching the Application

The Zoom SDK includes modified Qt libraries that are essential for proper execution.  
Use the following command to launch your Rust project, ensuring the modified Qt libraries are included in the library path:

```bash
LD_LIBRARY_PATH=zoom-meeting-sdk-linux/qt_libs/Qt/lib:$LD_LIBRARY_PATH cargo run
```

With these steps, you should be able to compile and run your Rust application with the Zoom SDK on Linux.  
Enjoy integrating advanced video conferencing features with Rust!

---

## Modifications to regenerate bindings with bindgen

When regenerating the bindings using `bindgen`, you need to make the following modification:

In the files:

- `meeting_ai_companion_interface.h`
- `meeting_chat_interface.h`

Add the following include:

```c
#include <ctime>
```

---

Happy coding!
