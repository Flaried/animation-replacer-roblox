# 🎭 Animation Replacer for Roblox
*Automatic Animation Spoofer*

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Roblox](https://img.shields.io/badge/Roblox-00A2FF?style=for-the-badge&logo=roblox&logoColor=white)](https://www.roblox.com/)

---

## 🚀 Overview

**Animation Spoofing** is an automated tool designed to fix the handling of third part animations in your Roblox game. This bot scrapes Roblox files, spoofs animations, and republishes them to ensure functionality when you publish your game.

### ✨ Key Features

- 🔄 **Automatic Animation Spoofing** - No manual intervention required
- 🛡️ **Uses .ROBLOSECURITY Cookie** - Works directly with your Roblox cookie authentication
- 🎯 **Minimum Configuration** - Just provide info like cookie and group id and let it work

---

### 📝 Development Status
❌Core functionality implemented

---
## 🛠️ Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- Valid Roblox authentication cookie
- .RBLX or Studio file with animations that aren't published by your account

### 📦 Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/yourusername/animation-replacer-roblox.git
   cd animation-replacer-roblox
   ```

2. **Build the project**
   ```bash
   cargo build --release
   ```

3. **Run the application**
   ```bash
   cargo run
   ```

---

## 🎯 How It Works
1. **Authentication** - Uses the provided cookie for publishing APIs
2. **File Scanning** - The bot automatically scans and identifies animations in the Roblox files
3. **Spoofing** - Each animation is processed and republished to ensure compatbility
5. **Completion** - Your game has working animations.
---

## ⚙️ Configuration
The tool requires minimal setup:
- **Roblox Cookie (REQUIRED)**: Your authentication token for accessing Roblox services
- **Target Files (Optional)**: Automatically scan root directory if no --target flag
- **Group Id (Optional)**: Upload to a group with --group flag
---

## 🚨 Important Notes

### Security & Privacy
- Your Roblox cookie is only used for authentication purposes
- Use at your own discretion and follow Roblox's Terms of Service
- Animations wont work if you dont use --group flag and then upload the game to a group

### Disclaimer
- This tool is provided as-is for educational and development purposes
- Users are responsible for compliance with Roblox's policies
- Always ask/give credit to animators.

---

## 🤝 Contributing

This is my first Rust project, I'm still learning Rust. So any contributions or suggestions will be accepted.

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙋‍♂️ Support

If you encounter any issues or have questions:

1. Check the [Issues](../../issues) page for existing solutions
2. Create a new issue with detailed information
3. Provide relevant error messages and system information

---

<div align="center">

**⭐ If this project helped you, consider giving it a star! ⭐**

</div>
