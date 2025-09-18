
你提供的内容是一个基于 ​​Protocol Buffers（protobuf）语法（proto3）​​ 定义的 ​​Rust / 跨平台远程桌面 / 远程控制协议消息结构（.proto 文件）​​，大概率来自一个类似 ​​RustDesk​​（或自研远程控制软件）项目的通信协议层。

✅ 一、总体概览（What is this?）
这是一个 ​​.proto 文件​​，用于定义客户端与服务器（或客户端之间）在远程控制 / 远程桌面会话中交互所使用的 ​​所有消息类型（Message Types）和数据结构（Data Structures）​​。

它被用于：

​​序列化 / 反序列化 网络数据​​（通过 protobuf 编译器生成对应语言的代码，如 Rust / TypeScript / C++ 等）

​​定义远程会话中传输的各种指令、事件、媒体数据、控制信号等​​

​​作为通信双方（如控制端 / 被控端）之间的“契约”或“接口”​​

所属项目极有可能是一个 ​​跨平台、低延迟、功能完备的远程桌面软件​​，支持如下功能：

功能类别

支持情况（从 proto 推断）

🖥️ 远程桌面画面传输

✅（支持多种编码：VP8/VP9/AV1/H264/H265/RGB/YUV）

⌨️ 键盘输入

✅（KeyEvent，支持多种键盘模式与跨平台键位映射）

🖱️ 鼠标输入

✅（MouseEvent、PointerDeviceEvent、Touch 事件）

📷 剪贴板同步

✅（Clipboard，支持多种格式如文本、图片、富文本）

🖼️ 文件传输

✅（FileAction、FileTransfer、ReadDir 等复杂文件操作）

📺 屏幕/显示器管理

✅（DisplayInfo、SwitchDisplay、SupportedResolutions）

🔒 认证与安全

✅（LoginRequest、Auth2FA、PublicKey、Hash、PeerInfo）

🎤 音频传输

✅（AudioFrame、AudioFormat）

📞 语音通话

✅（VoiceCallRequest / Response）

💻 终端（Shell）

✅（Terminal、TerminalAction、OpenTerminal 等）

🛠️ 系统功能

✅（WindowsSessions、OSLogin、ElevationRequest、UAC 相关）

🔄 权限与选项

✅（OptionMessage、PermissionInfo、Misc）

📋 插件系统

✅（PluginRequest / PluginFailure）

📤 消息通知 / 弹窗

✅（MessageBox、BackNotification）

📸 截图

✅（ScreenshotRequest / Response）

🧠 性能监控 / 延迟测试

✅（TestDelay）

🧩 多媒体与编码协商

✅（CodecAbility、SupportedEncoding、SupportedDecoding）

🖐️ 触摸与手势（移动端？）

✅（TouchEvent、TouchScaleUpdate 等）

🧩 二、核心消息类型（重要 Message 分类整理）
为了便于理解，我将这份 proto 文件中的核心 Message 按功能模块进行分类介绍 👇：

🖥️ 1. 远程桌面画面（Video / Frame / Encoding）
Message

用途

VideoFrame

核心视频帧容器，使用 oneof支持多种编码格式（如 VP8、VP9、H264、H265、AV1、RGB、YUV）

EncodedVideoFrame

编码后的视频帧数据（含 pts 时间戳、是否为关键帧）

EncodedVideoFrames

一组编码帧（比如一帧组或一个包）

YUV/ RGB

原始像素格式传输（适用于低延迟或特殊编码需求）

CodecAbility/ SupportedEncoding/ SupportedDecoding

客户端声明自己支持的编解码能力（比如是否支持 AV1、I444 等）

ImageQuality

图像质量级别（Low / Balanced / Best）

TestDelay

用于测试网络延迟与带宽

⌨️ 2. 输入控制（键盘 / 鼠标 / 触摸）
Message

用途

KeyEvent

键盘按下 / 释放事件，支持多种键位类型（ControlKey、Unicode、虚拟键码等）

MouseEvent

鼠标移动、点击事件，带修饰键（modifiers）

PointerDeviceEvent/ TouchEvent

触摸事件（移动端支持？缩放、平移等）

KeyboardMode

键盘输入模式（Legacy / Map / Translate / Auto）

ControlKeyenum

枚举所有可能的控制键，如 Enter、Alt、F1~F12、CtrlAltDel、VolumeUp 等

📷 3. 剪贴板同步
Message

用途

Clipboard

普通剪贴板内容（文本、图片、HTML、RTF、特殊格式等）

MultiClipboards

多剪贴板支持（比如移动端或高级场景）

ClipboardFormatenum

剪贴板数据格式（Text、Rtf、Html、ImagePng、ImageSvg、Special）

📁 4. 文件传输与管理
Message

用途

FileAction

文件操作统一入口（如读目录、传输文件、删文件、重命名等）

FileTransfer

文件传输控制（发送 / 接收请求、取消、确认等）

FileTransferBlock

文件分块数据（支持压缩）

FileTransferError/ FileTransferDone/ FileTransferDigest

传输状态反馈

ReadDir/ FileEntry/ FileDirectory

文件系统目录读取相关

FileTypeenum

文件类型（File、Dir、Link 等）

🖥️ 5. 显示器 / 屏幕管理
Message

用途

DisplayInfo

显示器信息（分辨率、位置、名称、是否在线、缩放比例等）

SwitchDisplay

切换被控端显示目标（比如多屏切换）

SupportedResolutions/ Resolution

支持的分辨率列表

CaptureDisplays/ ToggleVirtualDisplay

虚拟显示器控制

DisplayResolution

指定显示器的分辨率设置

👤 6. 用户认证与登录
Message

用途

LoginRequest

登录请求（含用户名、密码、hwid、option、2FA、终端、文件传输等联合字段）

LoginResponse

登录响应（成功返回 PeerInfo，失败返回错误信息）

PeerInfo

被控端信息（用户、平台、显示器、功能、编码支持、版本等）

Auth2FA

双因素认证

IdPk

ID 与公钥（可能用于 P2P 或加密通信）

OSLogin

操作系统账户登录（如 Windows 域账户）

🔐 7. 安全与加密
Message

用途

PublicKey

非对称加密公钥（可能用于 P2P 信道加密）

SignedId

签名后的身份标识

Hash

用于挑战-响应认证（含 salt 和 challenge）

🎤 8. 音频
Message

用途

AudioFrame

音频数据帧

AudioFormat

音频格式（采样率、声道数等）

📞 9. 语音通话（可选功能）
Message

用途

VoiceCallRequest/ VoiceCallResponse

发起 / 响应语音通话

💻 10. 终端（Shell / 命令行）
Message

用途

Terminal/ TerminalAction

打开 / 发送数据 / 调整大小 / 关闭终端

TerminalResponse

终端状态反馈（如打开成功、收到数据、关闭、错误等）

OpenTerminal/ ResizeTerminal/ TerminalData/ CloseTerminal

终端操作细分

🛠️ 11. 系统功能与权限
Message

用途

OptionMessage

客户端功能选项（如是否启用隐私模式、文件传输、音频、录屏等）

PermissionInfo

权限开关（如键盘、剪贴板、文件、录屏、重启等）

Misc

杂项控制消息（如刷新视频、重启远端、UAC提权、插件、截屏等）

ElevationRequest

UAC / 管理员权限提权请求

WindowsSessions

Windows 用户会话列表（用于选择登录目标）

📨 12. 消息通知 / 弹窗 / 交互
Message

用途

MessageBox

通知对方弹出一个消息框（带标题、内容、链接等）

BackNotification

反馈某种状态（如隐私模式开启结果、阻止输入状态等）

🧩 13. 其它 / 插件 / 协议工具
Message

用途

PluginRequest/ PluginFailure

插件系统支持

MessageQuery

查询类消息（如查询某个显示器）

Cliprdr*一系列消息

剪贴板 Redirection 协议相关（RDP 协议兼容？）

🧩 三、通用结构与工具类型
Message / Type

用途

EncodedVideoFrames

一组编码帧的集合

oneof union

Proto3 中的经典用法，用于表示“多选一”的消息类型（如 VideoFrame 中支持多种编码格式）

enum

枚举类型，如 KeyboardMode、ClipboardFormat、FileType、ControlKey等

bytes

用于承载任意二进制数据（如编码帧、音频帧、文件块等）

repeated

表示数组 / 列表（如 repeated FileEntry、repeated DisplayInfo）

Timestamp / Instant相关

通过 int64 pts或 uint64 id等自行实现

✅ 四、总结一句话
该 .proto文件是一个功能极为丰富、结构清晰的 ​​远程桌面控制协议定义​​，使用 ​​protobuf（proto3）​​ 编写，定义了从 ​​视频流、音频、输入控制、剪贴板、文件传输、终端、系统权限、多显示器管理、安全认证 到 插件与通知系统​​ 等几乎所有远程控制场景下的通信消息类型，是整个远程桌面软件的​​通信中枢与数据契约​​。

✅ 五、如果你想进一步…
目标

我可以帮助你

🧩 ​​生成代码​​

告诉我目标语言（如 Rust / TypeScript / Python / C++），我可以指导如何用 protoc生成对应代码

📦 ​​模块拆分建议​​

如果 proto 文件过于庞大，我可以帮你按功能拆分成多个 .proto 文件（如 video.proto、input.proto、file.proto）

🛡️ ​​安全增强建议​​

比如如何保护 Login / Auth / Clipboard 等敏感消息

🌐 ​​跨平台兼容性​​

比如如何处理 Windows / Linux / macOS 差异

📈 ​​性能优化​​

比如视频编码选择策略、帧率控制、压缩选项等

🧪 ​​单元测试 / Mock 数据​​

帮你为每种 Message 类型构造测试数据

