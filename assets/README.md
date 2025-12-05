# Assets Directory

## Icon File

To add a custom icon to your Windows executable:

1. Create or download an `.ico` file
2. Place it in this directory as `icon.ico`
3. Rebuild with `cargo build --release`

The icon will be embedded into the `.exe` file and will appear in Windows Explorer.

### Creating an ICO file

You can:
- Use online tools like https://convertio.co/png-ico/ or https://www.icoconverter.com/
- Use image editing software like GIMP or Photoshop
- Convert a PNG to ICO using tools

The recommended size is 256x256 pixels for best quality.

