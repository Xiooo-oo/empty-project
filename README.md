# 咪咪桌宠

把家里的黑白奶牛猫咪咪，变成一只会待机、散步和打盹的 Windows 像素桌宠。

![咪咪桌宠动作演示](docs/demo.gif)

## 功能

- 透明、无边框、始终置顶的小窗口；非猫咪轮廓区域会把鼠标点击交给桌面上的其他窗口。
- 左键按住猫咪可拖动。右键菜单可让咪咪眨眼、跳跃、喵叫、散步，或者切换休息状态。
- 系统托盘右键菜单可切换休息状态或退出；左键单击托盘也能切换休息。
- 通过 Windows 全局空闲输入检测挂机；连续 3 分钟没有键盘或鼠标输入时，咪咪会趴下睡觉。
- GIF 格式的中文 README 演示图，MIT 许可证。

## Windows 开发

需要 Windows 10/11、Node.js 20.19+ 或 22.12+、Rust stable 工具链，以及 WebView2 Runtime。首次构建需要联网下载 npm 与 Cargo 依赖。

```powershell
npm install
npm run tauri dev
```

仅预览猫咪、动画和右键动作菜单（不启动 Rust/Tauri）：

```powershell
npm run dev
```

然后打开终端显示的本机网址，通常为 `http://127.0.0.1:1420`。浏览器预览不包含系统托盘、窗口置顶、系统级挂机检测或桌面点击穿透。

生成 Windows 安装包：

```powershell
npm run tauri build
```

安装包位于 `src-tauri/target/release/bundle/`，包含 NSIS 安装程序和 MSI。

## GitHub 云端构建

仓库包含 `.github/workflows/windows-build.yml`。把代码推送到 `main` 后，GitHub Actions 会在 Windows 云端生成 x64 安装包；也可以在仓库的 **Actions → Windows build → Run workflow** 手动启动。完成后从运行页面的 **Artifacts** 下载。

此工作流生成的是未签名安装包：它方便获取构建结果，但不能保证在启用了 Smart App Control 的电脑上安装或运行。公开发布时，建议配置来自微软受信任根证书计划中 CA 的 RSA 代码签名证书，签署应用所需的所有二进制文件和安装包，并在启用 Smart App Control 的 Windows 设备上验证。证书私钥和密码应仅保存在 GitHub Actions Secrets 中，绝不能提交到仓库。

## 项目结构

```text
src/                  Svelte 桌宠界面和动作
src-tauri/            Tauri 窗口、托盘及 Windows 输入监测
public/cats/           去背景后的猫咪透明 PNG 帧
assets/source/         原始猫咪像素图
docs/demo.gif          动作预览
scripts/prepare_assets.py  从原图重做透明帧、演示 GIF 和应用图标
```

桌宠动作使用提供的坐姿、趴睡姿态和三张行走姿态。眨眼、跳跃、呼吸动画由界面驱动；喵叫使用浏览器音频合成，因此不需要额外音频文件。素材脚本优先读取仓库里的 `assets/source/`；首次从新原图生成时，可在 PowerShell 里设置 `$env:CAT_DESKPET_SOURCE` 指向图片目录，然后运行 `python scripts/prepare_assets.py`。

## 素材与许可

程序代码、文档与本仓库中的桌宠素材按 MIT 许可证发布，详见 [LICENSE](LICENSE)。发布到公开仓库前，请确认原图及其衍生素材适用于你的发布场景。
