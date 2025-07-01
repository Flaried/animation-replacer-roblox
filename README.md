# 🎭 Animation Replacer for Roblox (WIP)
*Automatic Animation Replacer*

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Roblox](https://img.shields.io/badge/Roblox-00A2FF?style=for-the-badge&logo=roblox&logoColor=white)](https://www.roblox.com/)



---
>    ⚠️ Disclaimer: In active development, so it wont replace animation instances in the game, only the animation Ids in the scripts.

## 📝 Development Status
- ✅ Scrape animations in lua scripts
- ✅ Scrape animation objects in the game file
- ✅ Fetch animation metadata, file contents, and asset types
- ✅ Upload multiple animations in a concurrent system; using [semaphore](https://docs.rs/semaphore/latest/semaphore/)
- ✅ Writing animations back to script source 
- ✅ Flags and user configuration for easy use
- ❌ Replace the animation instances in-game (Only replaces scripts for now)  
- ❌ Rename the Animations as the same as the ones it replaces (Requires extra API calls for scripts)




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
> 🪟 For Windows users heres: [cargo install guide](https://doc.rust-lang.org/cargo/getting-started/installation.html) (to fix cargo not working)

3. **Run the application**

To run the tool, you’ll need your Roblox ``.ROBLOSECURITY`` cookie.
This is required to authenticate your account for uploading animations.

>    ⚠️ Important: Never share your Roblox cookie. It grants full access to your account.
>    If you're unsure how to retrieve it, here’s a tutorial:
>    [How to get your Roblox cookie (YouTube)](https://www.youtube.com/watch?v=zkSnBV7oOZM)

If you're concerned about using your main account, consider creating an alternate account, adding it as an admin to your group, and uploading from there.

   ```bash
   cargo run -- --cookie "COOKIEHERE" --file "example.rbxl"
   ```

> ⚠️ Tip:
>
> If you're uploading the game through a group, be sure to include the ``--group "GROUP_ID"`` flag.
>
> Recommended to use the ``--ouput "ouput.rbxl"`` flag to avoid data loss if the game corrupts. 
<!-- > ⚠️ This project is currently under active development.   -->
<!-- > Installation instructions will be provided in a future release. ⚠️ -->
---


### 🐕 Overview (BETA)

**Animation Spoofing/Reuploading** is an automated process designed to fix the handling of third party animations in your Roblox game. This program will scrape Roblox files, spoofs animations, and republish them to ensure functionality when you publish your game.

---
### Upcoming Key Features

- **Automatic Animation Spoofing** - No manual intervention required and all through the terminal
- **Group Support** - Supports Group publishing

- **Uses .ROBLOSECURITY Cookie** - Works directly with your Roblox cookie authentication
- **Minimum Configuration** - Just provide info like cookie and group id and let it work
-  **Fast and Efficient** - This program will be completely free, and will be as efficient as possible.
---


## 🛠️ Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- Valid roblox authentication cookie
- .RBLX or Studio file with animations

## 💻 How It Works
2. **File Scanning** - The bot automatically scans and identifies animations in the Roblox files
3. **Reuploading** - Each animation is processed and republished to ensure compatbility
5. **Completion** - Your game has working animations.
---

## ⚙️ Configuration
The tool requires minimal setup:
- **Roblox Cookie (REQUIRED)**: Your authentication token for accessing Roblox services
- **Target Files (Optional)**: Automatically scan root directory if no --target flag
- **Group Id (Optional)**: Upload to a group with --group flag
---

## 🚨 Important Notes

### Disclaimer
- Users are responsible for compliance with Roblox's policies
- Always ask/give credit to animators.
- Animations wont work if you dont use --group flag and then upload the game to a group
- Your Roblox cookie is only used for authentication purposes


 - This is my first Rust project, I'm still learning Rust. So any contributions or suggestions will be accepted.

---

## 🤝 Credit 
Im using a roblox wrapper; [Roboat](https://github.com/fekie/roboat) to achieve a more stable and better development with roblox's changes.

For researching rust ive been using the official [rust book](https://doc.rust-lang.org/book/).

having AI (Claude) only help with ONLY the readme, lifetimes, and some refactors for optimization, as this is an educational project for me.


---

<div align="center">

**⭐ If this project helped you, consider giving it a star! ⭐**

</div>
