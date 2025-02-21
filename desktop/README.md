### Linux Desktop Icon

To create a desktop shortcut for the application on Linux, follow these steps:

1. **Copy the Icon**: Place the `epoch.png` icon in the appropriate directory for application icons.
  ```bash
  sudo cp epoch.png /usr/share/icons/hicolor/256x256/apps/
  ```

2. **Copy the Desktop Entry**: Copy the `epoch.desktop` file to your desktop.
  ```bash
  cp desktop/epoch.desktop ~/Desktop
  ```

3. **Make the Desktop Entry Executable**: Ensure the desktop entry file is executable.
  ```bash
  chmod +x ~/Desktop/epoch.desktop
  ```

The `epoch.desktop` file should contain the following content:
```ini
[Desktop Entry]
Version=1.0
Type=Application
Name=Epoch
Icon=/usr/share/icons/hicolor/256x256/apps/epoch.png
Exec=/path/to/your/application
Comment=Start the Epoch application
Terminal=false
Categories=Utility;
```

Replace `/path/to/your/application` with the actual path to the executable file of your application.


