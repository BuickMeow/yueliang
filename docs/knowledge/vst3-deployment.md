# VST3 插件部署指南（macOS）

## 问题

Rust + nih-plug 编译生成的 `.dylib` 文件无法被 DAW 直接识别。

## 原因

macOS 上的 VST3 插件必须是**标准 Bundle 结构**，不能是单个动态库文件。

## 正确的包结构

```
PluginName.vst3/                    # 插件包（文件夹）
└── Contents/
    ├── Info.plist                  # 包信息配置文件
    └── MacOS/
        └── PluginName              # 可执行文件（无扩展名）
```

## 创建步骤

### 1. 创建目录结构

```bash
PLUGIN_NAME="Yueliang"
mkdir -p ~/Library/Audio/Plug-Ins/VST3/${PLUGIN_NAME}.vst3/Contents/MacOS
```

### 2. 复制编译产物

```bash
cp target/release/lib${PLUGIN_NAME}.dylib \
   ~/Library/Audio/Plug-Ins/VST3/${PLUGIN_NAME}.vst3/Contents/MacOS/${PLUGIN_NAME}
```

注意：目标文件名**没有扩展名**。

### 3. 创建 Info.plist

```bash
cat > ~/Library/Audio/Plug-Ins/VST3/${PLUGIN_NAME}.vst3/Contents/Info.plist << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>English</string>
    <key>CFBundleExecutable</key>
    <string>Yueliang</string>
    <key>CFBundleIdentifier</key>
    <string>com.jieneng.yueliang</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>Yueliang</string>
    <key>CFBundlePackageType</key>
    <string>BNDL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleVersion</key>
    <string>0.1.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF
```

关键字段：
- `CFBundleExecutable`：必须与 MacOS 目录下的文件名一致
- `CFBundleIdentifier`：反向域名格式，唯一标识

## DAW 扫描

1. 打开 DAW 的插件管理/设置
2. 重置插件目录（Reset Plugin Catalog）
3. 重新扫描（Scan Now）
4. 在 VST3 分类下查找插件

## 安装位置

| 位置 | 说明 |
|------|------|
| `~/Library/Audio/Plug-Ins/VST3/` | 用户级，推荐 |
| `/Library/Audio/Plug-Ins/VST3/` | 系统级，需要管理员权限 |

## 参考

- [VST3 Documentation - macOS Bundle](https://developer.steinberg.help/display/VST/Plug-in+Format+Structure)
