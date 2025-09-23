// ==================== 标准库模块导入 ====================
use std::{
    collections::{HashMap, HashSet},  // HashMap: 键值对集合；HashSet: 唯一值集合
    fs,                               // 文件读写相关操作
    io::{Read, Write},                // 读写 trait，用于处理输入输出流
    net::{                            // 网络相关类型定义
        IpAddr,                       // IP 地址
        Ipv4Addr,                     // IPv4 地址
        Ipv6Addr,                     // IPv6 地址
        SocketAddr,                   // 套接字地址（IP + Port）
    },
    ops::{Deref, DerefMut},           // 用于智能指针的解引用操作
    path::{Path, PathBuf},            // 文件路径类型：Path 不可变，PathBuf 可变
    sync::{Mutex, RwLock},            // 线程同步：Mutex（互斥锁）、RwLock（读写锁）
    time::{                           // 时间相关
        Duration,                     // 时间段，如 2秒 = Duration::from_secs(2)
        Instant,                      // 高精度时间点，用于计时
        SystemTime,                   // 系统时间
    },
};


// ==================== 第三方库导入 ====================
use anyhow::Result;                   // 简化错误处理的 Result 类型
use bytes::Bytes;                     // 高效字节缓冲区类型
use rand::Rng;                        // 随机数生成
use regex::Regex;                     // 正则表达式支持
use serde as de;                      // 序列化框架（别名为 de）
use serde_derive::{Deserialize, Serialize}; // 派生宏：自动生成 Serialize/Deserialize
use serde_json;                       // JSON 序列化/反序列化库
use sodiumoxide::base64;              // libsodium 提供的 Base64 编解码
use sodiumoxide::crypto::sign;        // 数字签名相关功能



// ==================== 本地模块导入 ====================
use crate::{
    compress::{compress, decompress}, // 数据压缩与解压函数
    log,                              // 日志模块
    password_security::{              // 密码安全模块
        decrypt_str_or_original,      // 解密字符串（失败返回原串）
        decrypt_vec_or_original,      // 解密字节数据（失败返回原数据）
        encrypt_str_or_original,      // 加密字符串（失败返回原串）
        encrypt_vec_or_original,      // 加密字节数据（失败返回原数据）
        symmetric_crypt,              // 对称加密功能
    },
};

// ==================== 全局常量定义 ====================
pub const RENDEZVOUS_TIMEOUT: u64 = 12_000;   // 集结/协商超时：12 秒（单位毫秒）
pub const CONNECT_TIMEOUT: u64 = 18_000;      // 连接超时：18 秒
pub const READ_TIMEOUT: u64 = 18_000;         // 读取超时：18 秒

// QUIC 推荐 NAT 保活间隔为 15 秒，见相关链接
pub const REG_INTERVAL: i64 = 15_000;         // 心跳/注册间隔：15 秒（单位毫秒）

pub const COMPRESS_LEVEL: i32 = 3;            // 压缩级别：推荐 3（速度与压缩比平衡）

const SERIAL: i32 = 3;                        // 序列化版本号（用途需结合代码逻辑）
const PASSWORD_ENC_VERSION: &str = "00";      // 密码加密版本标识，用于兼容性

pub const ENCRYPT_MAX_LEN: usize = 128;       // 敏感信息（如密码/PIN）最大加密长度（字节）

//📌 1. 常量定义（与网络保活、压缩、加密相关）

// 以下常量定义来源于 QUIC 协议相关讨论与建议：
// - QUIC 官方草案推荐 NAT 保活间隔为 15 秒
// - 有人建议 25 秒，但最终采用 15 秒
// - 相关链接见注释上方（quic-go 与 ietf 草案）
// 15 秒是推荐的 NAT 穿透保活时间，用于维持连接不断开
pub const REG_INTERVAL: i64 = 15_000;  // 单位：毫秒（即 15 秒），用于注册或心跳包发送间隔

pub const COMPRESS_LEVEL: i32 = 3;     // 数据压缩级别，范围通常为 0（无压缩）~ 9（最高压缩），3 是平衡性能与压缩率的推荐值

const SERIAL: i32 = 3;                 // 序列号 / 版本号，可能用于数据结构版本控制、配置版本等

const PASSWORD_ENC_VERSION: &str = "00"; // 密码加密版本标识，用于标识当前使用的加密算法版本，便于兼容旧版本

pub const ENCRYPT_MAX_LEN: usize = 128;  // 最大加密长度（单位：字节），用于密码、PIN 等敏感信息，超出部分可能不加密
                                           // 注意：该限制仅适用于特定数据，不是全部数据都受此限制
//✅ 作用：定义了程序中与 ​​网络保活、数据压缩、加密安全​​ 相关的常量参数，是全局共享的配置值。


//📌 2. 平台相关的静态变量（仅 macOS）

// 仅在 macOS 平台编译时生效
#[cfg(target_os = "macos")]
lazy_static::lazy_static! {
    // 定义一个全局、线程安全的字符串，表示当前应用的 Bundle Identifier（组织名 + 应用名）
    // 这在 macOS 上常用于权限、沙盒、签名相关用途
    pub static ref ORG: RwLock<String> = RwLock::new("com.carriez".to_owned());
}
//✅ 作用：为 macOS 平台定义了一个全局的组织标识符（类似 iOS 的 Bundle ID），可能是用于权限控制或应用签名。使用了 lazy_static延迟初始化 + RwLock保证线程安全。

//📌 3. 类型别名（提高代码可读性）

type Size = (i32, i32, i32, i32);   // 定义一个类型别名 Size，表示一个四元组 (i32, i32, i32, i32)
                                    // 可能用于表示屏幕分辨率、窗口大小、位置等（x, y, w, h ?)
type KeyPair = (Vec<u8>, Vec<u8>);  // 定义一个类型别名 KeyPair，表示一对向量（通常是公钥和私钥）
                                    // 用于加密通信或身份认证
//✅ 作用：给普通的元组类型起了语义化的别名，让代码更清晰，比如 Size比 (i32, i32, i32, i32)更直观。


//📌 4. 全局共享状态（使用 lazy_static + RwLock / Mutex）
// > 这是该代码段最核心的部分：​​✅定义了一组全局的、✅延迟初始化的、✅线程安全的配置和状态对象​​，它们在整个程序运行期间可能被多个线程访问，比如：
//    -程序配置（Config）
//    -本地配置（LocalConfig）
//    -在线设备状态（ONLINE）
//    -可信设备列表（TRUSTED_DEVICES）
//    -当前状态（STATUS）
//    -服务器地址（PROD_RENDEZVOUS_SERVER 等）
//    -用户默认设置、覆盖设置、显示设置等
//🔄 lazy_static 简介（如果你不熟悉）
//    -lazy_static::lazy_static!是一个 Rust 宏，用于定义​​延迟初始化的静态变量​​。
//    -由于 Rust 的静态变量要求必须是编译期可知的常量，而像 Config::load()是运行时才能初始化的，因此需要 lazy_static。
//    -结合 RwLock或 Mutex，可以实现​​多线程安全访问​​。
//✅ 通用配置相关（RwLock<Config> 等）
lazy_static::lazy_static! {
    static ref CONFIG: RwLock<Config> = RwLock::new(Config::load());            // 全局共享的 Config 配置，使用 RwLock 允许多个线程同时读，写时独占
    static ref CONFIG2: RwLock<Config2> = RwLock::new(Config2::load());        // 全局共享的 Config2 配置（可能是另一种配置结构，比如高级设置）
    static ref LOCAL_CONFIG: RwLock<LocalConfig> = RwLock::new(LocalConfig::load());    // 全局共享的 LocalConfig（可能是本地个性化配置，如语言、主题）
    static ref STATUS: RwLock<Status> = RwLock::new(Status::load());    // 全局共享的状态信息（如连接状态、运行状态等）
    static ref TRUSTED_DEVICES: RwLock<(Vec<TrustedDevice>, bool)> = Default::default();    // 可信设备列表，包含设备信息和一个布尔值（可能表示是否已更新/加载）
    static ref ONLINE: Mutex<HashMap<String, i64>> = Default::default();            // 当前在线的用户/设备，用 HashMap<String, i64> 表示，可能是 device_id -> 最后心跳时间戳
    //✅ 作用：这些变量保存了程序运行时需要的​​核心配置和状态信息​​，使用 RwLock或 Mutex保证线程安全，用 lazy_static延迟加载。

    
    //🛰️ 服务器 / 应用信息相关
    pub static ref PROD_RENDEZVOUS_SERVER: RwLock<String> = RwLock::new("".to_owned());        // 生产环境默认的中继服务器地址（字符串，可被修改）
    pub static ref EXE_RENDEZVOUS_SERVER: RwLock<String> = Default::default();            // 当前实际使用的中继服务器地址（可能是动态更新的）
    pub static ref APP_NAME: RwLock<String> = RwLock::new("RustDesk".to_owned());            //应用名称（如 "RustDesk"），可能是用于显示或日志
    //✅ 作用：定义了与 ​​服务器地址、应用名称​​ 相关的全局变量，通常是动态配置的。

    
    static ref KEY_PAIR: Mutex<Option<KeyPair>> = Default::default();            // 当前程序的密钥对（可能是非对称加密的公钥/私钥），类型是 Vec<u8> 的元组✅ 作用：存储当前设备的加密密钥对，用 Mutex保证线程安全，初始值为 None。

    //🧩 用户默认配置与覆盖配置
    // 用户默认配置 + 最后加载时间
    static ref USER_DEFAULT_CONFIG: RwLock<(UserDefaultConfig, Instant)> = RwLock::new((UserDefaultConfig::load(), Instant::now()));
    
    pub static ref NEW_STORED_PEER_CONFIG: Mutex<HashSet<String>> = Default::default();        // 新存储的对等端（peer）配置（HashSet<String>），可能是设备 ID 等

    // 默认设置 / 覆盖设置 / 显示设置 / 本地设置 等，都是键值对形式的配置（HashMap<String, String>）
    pub static ref DEFAULT_SETTINGS: RwLock<HashMap<String, String>> = Default::default();
    pub static ref OVERWRITE_SETTINGS: RwLock<HashMap<String, String>> = Default::default();
    pub static ref DEFAULT_DISPLAY_SETTINGS: RwLock<HashMap<String, String>> = Default::default();
    pub static ref OVERWRITE_DISPLAY_SETTINGS: RwLock<HashMap<String, String>> = Default::default();
    pub static ref DEFAULT_LOCAL_SETTINGS: RwLock<HashMap<String, String>> = Default::default();
    pub static ref OVERWRITE_LOCAL_SETTINGS: RwLock<HashMap<String, String>> = Default::default();
    pub static ref HARD_SETTINGS: RwLock<HashMap<String, String>> = Default::default();
    pub static ref BUILTIN_SETTINGS: RwLock<HashMap<String, String>> = Default::default();
    //✅ 作用：定义了非常丰富的配置存储结构，包括：
    //默认配置 vs 用户覆盖配置
    //普通设置、显示设置、本地化设置等
    //每个都用 HashMap<String, String>存储键值对，用 RwLock保证线程安全
}


lazy_static::lazy_static! {
    pub static ref APP_DIR: RwLock<String> = Default::default();        // 当前应用的数据目录 / 安装目录（字符串形式，延迟初始化）
}

// 仅在 Android / iOS 平台定义：应用主目录（可能是沙盒内路径）
#[cfg(any(target_os = "android", target_os = "ios"))]
lazy_static::lazy_static! {
    pub static ref APP_HOME_DIR: RwLock<String> = Default::default();
}




pub const LINK_DOCS_HOME: &str = "https://rustdesk.com/docs/en/";       // RustDesk 官方文档首页（英文）
pub const LINK_DOCS_X11_REQUIRED: &str = "https://rustdesk.com/docs/en/manual/linux/#x11-required";     // 如果使用 X11（Linux 桌面环境），需要查看的文档页面
pub const LINK_HEADLESS_LINUX_SUPPORT: &str =
    "https://github.com/rustdesk/rustdesk/wiki/Headless-Linux-Support";     // 有关 Linux 无头模式（headless，无图形界面）支持的 Wiki 文档
lazy_static::lazy_static! {
     // 键值对：关键词 -> 文档链接
    pub static ref HELPER_URL: HashMap<&'static str, &'static str> = HashMap::from([
        ("rustdesk docs home", LINK_DOCS_HOME), 
        ("rustdesk docs x11-required", LINK_DOCS_X11_REQUIRED),
        ("rustdesk x11 headless", LINK_HEADLESS_LINUX_SUPPORT),
        ]);
}
//✅ 作用：定义了 RustDesk 相关的​​官方文档、Linux 支持、无头模式部署​​等帮助页面链接，可能是用于：
//在 GUI 中提供“帮助”按钮跳转
//在日志 / 错误提示中引导用户查阅官方资料
//内部排障或部署指引


//📌 3. 字符集常量（用于生成字符串、验证码等）
// 数字字符集：'0' ~ '9'，可用于生成纯数字字符串
const NUM_CHARS: &[char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
// 混合字符集：数字 + 部分小写字母（去除了容易混淆的字母如 'l', 'o', 'z' 等）
// 可能用于生成随机密码、验证码、token 等
const CHARS: &[char] = &[
    '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k',
    'm', 'n', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];
//    ✅ 作用：预定义字符集合，通常用于：
//    随机生成字符串（如验证码、临时密码、邀请码）
//    构造 token、密钥部分字符
//    去除了容易与数字混淆的字母（比如 o 和 0，l 和 1），提升用户体验


//这是默认配置，现在进行修改103---113
//
// pub const RENDEZVOUS_SERVERS: &[&str] = &["rs-ny.rustdesk.com"];
// pub const RS_PUB_KEY: &str = "OeVuKk5nlHiXp+APNn0Y3pC1Iwpwn44JGqrQCsWqmBw=";
// 
// pub const RENDEZVOUS_PORT: i32 = 21116;
// pub const RELAY_PORT: i32 = 21117;
// pub const WS_RENDEZVOUS_PORT: i32 = 21118;
// pub const WS_RELAY_PORT: i32 = 21119;
// 
// ==================== 你修改后的服务器与密钥配置 ====================
// 中继/ID 服务器地址列表（这里只设置了一个）
pub const RENDEZVOUS_SERVERS: &[&str] = &["hbyx.myds.me"];
// 服务器的公钥（可能是用于身份验证 / TLS / 中继安全等）
pub const RS_PUB_KEY: &str = "sDml1I2V7HwrJg+IlLISdSqYT09fUtA030IJwb8lNps=";
// 各个服务的端口号定义
pub const RENDEZVOUS_PORT: i32 = 2116;
pub const RELAY_PORT: i32 = 2117;
pub const WS_RENDEZVOUS_PORT: i32 = 2118;
pub const WS_RELAY_PORT: i32 = 2119;
//✅ 作用：这些是 ​​RustDesk 客户端连接的核心网络配置​​，包括：
​​//ID 服务器（RENDEZVOUS_SERVERS）​​：用于设备发现、在线状态同步
​​//中继服务器（RELAY_PORT）​​：当 P2P 打洞失败时，用于流量转发
​​//WebSocket 端口​​：可能是为了支持浏览器或其他 WebSocket 客户端接入
​​//RS_PUB_KEY​​：可能是服务器的身份公钥，用于加密通信或身份验证

pub fn init_default_settings() {
    DEFAULT_SETTINGS.write().unwrap().insert("password".to_string(), "Bai21359869".to_string());
    // 固定密码 Config::set_permanent_password("Bai21359869");
    
    DEFAULT_SETTINGS.write().unwrap().insert("unlock_pin".to_string(), "0.369".to_string());
    // 固定PIN Config::set_unlock_pin("0.369");

    DEFAULT_SETTINGS.write().unwrap().insert("temporary-password-length".to_string(), "6".to_string());
    DEFAULT_SETTINGS.write().unwrap().insert("allow-numeric-one-time-password".to_string(), "Y".to_string());
    // 一次性密码相关
        // Config::set_option("temporary-password-length".to_string(), "6".to_string());
        // Config::set_option("allow-numeric-one-time-password".to_string(), "Y".to_string());
    DEFAULT_SETTINGS.write().unwrap().insert("verification-method".to_string(), "password,otp".to_string());
    // 如果有 verification-method 选项，允许同时用两种密码 Config::set_option("verification-method".to_string(), "password,otp".to_string());

    DEFAULT_SETTINGS.write().unwrap().insert("allow-remote-config-modification".to_string(), "Y".to_string());
    // 权限：允许远程修改配置 Config::set_option("allow-remote-config-modification".to_string(), "Y".to_string());

    DEFAULT_SETTINGS.write().unwrap().insert("enable-check-update".to_string(), "N".to_string());
    // 检查更新开关：不允许启动时检查 Config::set_option("enable-check-update".to_string(), "N".to_string());
}


//📌 5. 序列化辅助宏：serde_field_string
// 定义一个宏，用于简化处理带有默认值的字符串类型字段的 serde 反序列化逻辑
// 目的是：当字段为空字符串时，使用默认值，而不是报错或使用空内容
macro_rules! serde_field_string {
    ($default_func:ident, $de_func:ident, $default_expr:expr) => {
        fn $default_func() -> String {
            $default_expr
        }
         // 反序列化函数：从外部数据（如 JSON）中解析字符串，如果为空则返回默认值
        fn $de_func<'de, D>(deserializer: D) -> Result<String, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            let s: String =
                de::Deserialize::deserialize(deserializer).unwrap_or(Self::$default_func());
            if s.is_empty() {
                return Ok(Self::$default_func());
            }
            Ok(s)
        }
    };
}
//✅ 作用：定义了一个通用宏，用来简化 Rust 结构体中 ​​String 类型字段​​ 的反序列化逻辑，支持：
//自定义默认值
//空字符串自动回退到默认值
//常用于配置项，比如用户未设置时使用合理默认

//📌 6. 序列化辅助宏：serde_field_bool
// 定义一个宏，用于简化布尔类型字段的处理：包括默认值、自定义逻辑、反序列化
macro_rules! serde_field_bool {
    ($struct_name: ident, $field_name: literal, $func: ident, $default: literal) => {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]// 定义一个结构体，用于包装一个布尔值
        pub struct $struct_name {
            // 该字段从序列化数据中读取，如果为空则使用默认值
            #[serde(default = $default, rename = $field_name, deserialize_with = "deserialize_bool")]
            pub v: bool,
        }

        // 为该结构体实现 Default trait，指定默认值来源
        impl Default for $struct_name {
            fn default() -> Self {
                Self { v: Self::$func() }
            }
        }
        // 自定义方法，用于读取该配置项的实际布尔值（可能从本地配置 / 注册表等读取）
        impl $struct_name {
            pub fn $func() -> bool {
                UserDefaultConfig::read($field_name) == "Y"
            }
        }
        // 实现 Deref 和 DerefMut，让该结构体可以像 bool 一样直接使用 .v 或直接解引用
        impl Deref for $struct_name {
            type Target = bool;

            fn deref(&self) -> &Self::Target {
                &self.v
            }
        }
        impl DerefMut for $struct_name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.v
            }
        }
    };
}
//    ✅ 作用：定义了一个用于处理 ​​布尔类型配置项​​ 的通用结构体与逻辑，比如：
//    某个功能开关（如启用通知、启用暗黑模式）
//    支持从本地配置（如 Windows 注册表、配置文件）中读取当前值
//    通过 Deref让它用起来就像一个普通的 bool值一样自然


//✅ 作用：定义了当前网络连接的​​类型​​，用于控制 RustDesk 如何建立连接：
//Direct：尝试直接连接（P2P 打洞）
//ProxySocks：通过 SOCKS5 代理服务器连接（比如在公司防火墙后面）
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NetworkType {
    Direct,// 直连模式：尝试 P2P 直连，不经过代理或中继
    ProxySocks, // 使用 SOCKS5 代理进行连接
}


//✅ 作用：保存与​​用户身份、加密密钥、设备配对​​相关的核心信息，比如：
//设备唯一 ID（用于识别）
//加密使用的密码、盐值、密钥对
//密钥是否已被用户认可（安全相关）
//每个配对设备的密钥确认状态（可能是多设备同步）
//🔐 这些字段大多涉及 ​​身份安全与加密通信​​，是 RustDesk 安全架构中的重要组成部分。
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct Config {
    #[serde(
        default,
        skip_serializing_if = "String::is_empty",
        deserialize_with = "deserialize_string"
    )]
    pub id: String, // use  // 用户唯一标识符 / 设备 ID
    #[serde(default, deserialize_with = "deserialize_string")]
    enc_id: String, // store  // 存储用的加密 ID
    #[serde(default, deserialize_with = "deserialize_string")]
    password: String,  // 用户密码（可能是用于设备间配对或登录）
    #[serde(default, deserialize_with = "deserialize_string")]
    salt: String,   // 密码盐值，用于加密增强
    #[serde(default, deserialize_with = "deserialize_keypair")]
    key_pair: KeyPair, // sk, pk  // 密钥对（公钥 + 私钥），用于身份验证或加密通信
    #[serde(default, deserialize_with = "deserialize_bool")]
    key_confirmed: bool,  // 密钥是否已经被用户确认（比如首次配对后点击确认）
    #[serde(default, deserialize_with = "deserialize_hashmap_string_bool")]
    keys_confirmed: HashMap<String, bool>,  // 每个设备的密钥确认状态
}


//🧩 3. SOCKS5 代理配置结构体：Socks5Server
//✅ 作用：用于配置 RustDesk 客户端在需要时连接的 ​​SOCKS5 代理服务器信息​​，适用于网络受限环境。
#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub struct Socks5Server {
    #[serde(default, deserialize_with = "deserialize_string")]
    pub proxy: String,// SOCKS5 代理服务器地址（比如 IP:Port）
    #[serde(default, deserialize_with = "deserialize_string")]
    pub username: String, // 代理用户名（如有）
    #[serde(default, deserialize_with = "deserialize_string")]
    pub password: String,// 代理密码（如有）
}

// more variable configs
//🧩 4. 核心配置结构体 2：Config2（网络 / 选项 / 设备信任等）
//✅ 作用：保存与 ​​网络连接策略、设备信任、用户 PIN、代理、扩展选项​​ 相关的信息，是对 Config的补充。
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct Config2 {
    #[serde(default, deserialize_with = "deserialize_string")]
    rendezvous_server: String,              // ID 服务器地址（设备发现用）
    #[serde(default, deserialize_with = "deserialize_i32")]
    nat_type: i32,                          // NAT 类型（可能用于打洞策略）
    #[serde(default, deserialize_with = "deserialize_i32")]
    serial: i32,                            // 配置序列号 / 版本
    #[serde(default, deserialize_with = "deserialize_string")]
    unlock_pin: String,                     // 解锁 PIN 码（可能是设备本地锁屏）
    #[serde(default, deserialize_with = "deserialize_string")]
    trusted_devices: String,                // 可信设备列表（可能是序列化字符串）

    #[serde(default)]
    socks: Option<Socks5Server>,                // 可选的 SOCKS5 代理配置

    // the other scalar value must before this
    #[serde(default, deserialize_with = "deserialize_hashmap_string_string")]
    pub options: HashMap<String, String>,           // 其他杂项配置（键值对）
}



//🧩 5. 屏幕分辨率结构体：Resolution
//✅ 作用：表示一个屏幕或窗口的分辨率，通常用于远程桌面会话中的显示设置。
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct Resolution {
    pub w: i32,// 宽度
    pub h: i32,// 高度
}


//🧩 6. 最复杂配置结构体：PeerConfig（远程会话的所有功能选项！）
//✅ 作用：这是 ​​RustDesk 远程会话功能的“总配置”结构体​​，它控制了：
​​//界面与交互​​：如光标显示、滚轮模式、图像质量、只读模式等
​​//功能开关​​：如文件传输、剪贴板同步、音频、隐私模式、终端保持等
​​//安全与连接​​：如端口转发、键盘模式、鼠标行为
​​//多显示器与分辨率​​：远程多屏支持、自定义分辨率
​//额外数据​​：如 Flutter UI 配置、传输状态、设备信息等
//🔧 其中大量使用了 #[serde(flatten)]，表示将子结构体的字段​​平铺到当前结构体中​​，以简化序列化与配置管理。

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PeerConfig {
    #[serde(default, deserialize_with = "deserialize_vec_u8")]
    // 密码（字节格式，可能是用于临时会话）
    pub password: Vec<u8>,
    #[serde(default, deserialize_with = "deserialize_size")]
    // 窗口尺寸相关（当前、全屏、远程桌面等）
    pub size: Size,
    #[serde(default, deserialize_with = "deserialize_size")]
    pub size_ft: Size,
    #[serde(default, deserialize_with = "deserialize_size")]
    pub size_pf: Size,
    #[serde(
        default = "PeerConfig::default_view_style",
        deserialize_with = "PeerConfig::deserialize_view_style",
        skip_serializing_if = "String::is_empty"
    )]
    pub view_style: String,
    // 界面风格、滚动条、图像质量等 UI/UX 设置
    // Image scroll style, scrollbar or scroll auto
    #[serde(
        default = "PeerConfig::default_scroll_style",
        deserialize_with = "PeerConfig::deserialize_scroll_style",
        skip_serializing_if = "String::is_empty"
    )]
    pub scroll_style: String,
    #[serde(
        default = "PeerConfig::default_image_quality",
        deserialize_with = "PeerConfig::deserialize_image_quality",
        skip_serializing_if = "String::is_empty"
    )]
    pub image_quality: String,
    #[serde(
        default = "PeerConfig::default_custom_image_quality",
        deserialize_with = "PeerConfig::deserialize_custom_image_quality",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub custom_image_quality: Vec<i32>,
    // 各种功能开关（扁平化结构，用 flatten 表示直接内嵌字段）
    #[serde(flatten)]
    pub show_remote_cursor: ShowRemoteCursor,
    #[serde(flatten)]
    pub lock_after_session_end: LockAfterSessionEnd,
    #[serde(flatten)]
    pub terminal_persistent: TerminalPersistent,
    #[serde(flatten)]
    pub privacy_mode: PrivacyMode,
    #[serde(flatten)]
    pub allow_swap_key: AllowSwapKey,
    #[serde(default, deserialize_with = "deserialize_vec_i32_string_i32")]
    pub port_forwards: Vec<(i32, String, i32)>,// 端口转发规则
    #[serde(default, deserialize_with = "deserialize_i32")]
    pub direct_failures: i32,
    #[serde(flatten)]
    pub disable_audio: DisableAudio,
    #[serde(flatten)]
    pub disable_clipboard: DisableClipboard,
    #[serde(flatten)]
    pub enable_file_copy_paste: EnableFileCopyPaste,
    #[serde(flatten)]
    pub show_quality_monitor: ShowQualityMonitor,
    #[serde(flatten)]
    pub follow_remote_cursor: FollowRemoteCursor,
    #[serde(flatten)]
    pub follow_remote_window: FollowRemoteWindow,
    #[serde(
        default,
        deserialize_with = "deserialize_string",
        skip_serializing_if = "String::is_empty"
    )]

    // 鼠标、多显示器相关设置
    pub keyboard_mode: String,
    #[serde(flatten)]
    pub view_only: ViewOnly,
    #[serde(flatten)]
    pub show_my_cursor: ShowMyCursor,
    #[serde(flatten)]
    pub sync_init_clipboard: SyncInitClipboard,
    // Mouse wheel or touchpad scroll mode
    #[serde(
        default = "PeerConfig::default_reverse_mouse_wheel",
        deserialize_with = "PeerConfig::deserialize_reverse_mouse_wheel",
        skip_serializing_if = "String::is_empty"
    )]
    pub reverse_mouse_wheel: String,
    #[serde(
        default = "PeerConfig::default_displays_as_individual_windows",
        deserialize_with = "PeerConfig::deserialize_displays_as_individual_windows",
        skip_serializing_if = "String::is_empty"
    )]
    pub displays_as_individual_windows: String,
    #[serde(
        default = "PeerConfig::default_use_all_my_displays_for_the_remote_session",
        deserialize_with = "PeerConfig::deserialize_use_all_my_displays_for_the_remote_session",
        skip_serializing_if = "String::is_empty"
    )]
    pub use_all_my_displays_for_the_remote_session: String,
    #[serde(
        rename = "trackpad-speed",
        default = "PeerConfig::default_trackpad_speed",
        deserialize_with = "PeerConfig::deserialize_trackpad_speed"
    )]
    pub trackpad_speed: i32,

    #[serde(
        default,
        deserialize_with = "deserialize_hashmap_resolutions",
        skip_serializing_if = "HashMap::is_empty"
    )]
    

    pub custom_resolutions: HashMap<String, Resolution>,
    // 自定义分辨率、额外选项、Flutter UI 配置、传输信息等
    // The other scalar value must before this
    #[serde(
        default,
        deserialize_with = "deserialize_hashmap_string_string",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub options: HashMap<String, String>, // not use delete to represent default values
    // Various data for flutter ui
    #[serde(default, deserialize_with = "deserialize_hashmap_string_string")]
    pub ui_flutter: HashMap<String, String>,
    #[serde(default)]
    pub info: PeerInfoSerde,
    #[serde(default)]
    pub transfer: TransferSerde,
}


//🧩 1. 为 PeerConfig提供默认值
//✅ 作用：为 PeerConfig（控制远程会话的几乎所有功能和 UI 行为）提供​​合理的默认值​​，当用户没有特别配置时，使用这些默认行为。
//包括：
//图像质量、窗口大小
//安全选项（禁用音频、剪贴板、文件传输）
//功能开关（光标显示、滚轮、多显示器）
//传输与同步选项
//键盘、鼠标、UI 行为

impl Default for PeerConfig {
    fn default() -> Self {
        Self {
            password: Default::default(),                      // 会话密码（字节向量）
            size: Default::default(),                          // 屏幕尺寸
            size_ft: Default::default(),                       // 全屏尺寸？
            size_pf: Default::default(),                       // ？
            view_style: Self::default_view_style(),            // 视图样式（如窗口装饰风格）
            scroll_style: Self::default_scroll_style(),        // 滚动条样式
            image_quality: Self::default_image_quality(),      // 图像质量预设
            custom_image_quality: Self::default_custom_image_quality(), // 自定义图像质量数值
            show_remote_cursor: Default::default(),            // 是否显示远程光标
            lock_after_session_end: Default::default(),        // 会话结束后是否锁定本地电脑
            terminal_persistent: Default::default(),           // 终端会话是否保持
            privacy_mode: Default::default(),                  // 隐私模式（如禁用某些功能）
            allow_swap_key: Default::default(),                // 是否允许交换 Ctrl/Alt 等
            port_forwards: Default::default(),                 // 端口转发规则列表
            direct_failures: Default::default(),               // 直连失败次数统计
            disable_audio: Default::default(),                 // 是否禁用音频传输
            disable_clipboard: Default::default(),             // 是否禁用剪贴板同步
            enable_file_copy_paste: Default::default(),        // 是否启用文件复制粘贴
            show_quality_monitor: Default::default(),          // 是否显示传输质量监控
            follow_remote_cursor: Default::default(),          // 是否跟随远程鼠标
            follow_remote_window: Default::default(),          // 是否跟随远程窗口
            keyboard_mode: Default::default(),                 // 键盘输入模式
            view_only: Default::default(),                     // 是否只读模式（不能操作远程）
            show_my_cursor: Default::default(),                // 是否显示本地光标
            reverse_mouse_wheel: Self::default_reverse_mouse_wheel(), // 鼠标滚轮反向
            displays_as_individual_windows: Self::default_displays_as_individual_windows(), // 多显示器是否作为独立窗口
            use_all_my_displays_for_the_remote_session: Self::default_use_all_my_displays_for_the_remote_session(), // 是否将所有显示器用于远程会话
            trackpad_speed: Self::default_trackpad_speed(),    // 触控板/鼠标速度
            custom_resolutions: Default::default(),            // 自定义分辨率列表
            options: Self::default_options(),                  // 其他键值对选项
            ui_flutter: Default::default(),                    // Flutter UI 相关配置
            info: Default::default(),                          // 设备/会话信息
            transfer: Default::default(),                      // 文件传输信息
            sync_init_clipboard: Default::default(),           // 是否同步初始化剪贴板
        }
    }
}


//🧩 2. 辅助结构体：PeerInfoSerde 与 TransferSerde
//✅ 作用：用于 ​​序列化传输与设备信息​​，比如：
//PeerInfoSerde：保存远端主机的基本信息，可能用于 UI 显示
//TransferSerde：记录当前正在进行的文件传输任务（读/写）

#[derive(Debug, PartialEq, Default, Serialize, Deserialize, Clone)]
pub struct PeerInfoSerde {
    #[serde(default, deserialize_with = "deserialize_string")]
    pub username: String,// 远程用户名称
    #[serde(default, deserialize_with = "deserialize_string")]
    pub hostname: String,// 远程主机名
    #[serde(default, deserialize_with = "deserialize_string")]
    pub platform: String,// 远程操作系统平台（Windows/macOS/Linux）
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct TransferSerde {
    #[serde(default, deserialize_with = "deserialize_vec_string")]
    pub write_jobs: Vec<String>,// 当前写任务（文件传输）
    #[serde(default, deserialize_with = "deserialize_vec_string")]
    pub read_jobs: Vec<String>, // 当前读任务
}


//🧩 3. 获取在线设备状态（NAT 保活相关）
//✅ 作用：从全局的 ONLINE（一个线程安全的 HashMap<String, i64>，记录设备最后活跃时间）中，取出​​最后一个活跃的设备时间戳，作为“在线状态”参考​​。
//可用于判断某个对等设备是否“在线”或最近活跃。
#[inline]
pub fn get_online_state() -> i64 {
    *ONLINE.lock().unwrap().values().max().unwrap_or(&0)
}

//🧩 4. 平台相关路径修正函数：patch()
//✅ 作用：对某些特殊系统路径进行兼容性处理，比如：
//Windows 系统服务账户路径
//macOS 的配置文件夹差异
//Linux 下 root 用户的路径回退逻辑

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn patch(path: PathBuf) -> PathBuf {
    // 仅在非移动端平台执行
    // Windows: 替换系统目录为服务账户目录
    // macOS: 替换 Application Support 为 Preferences
    // Linux: 如果是 root 用户，尝试获取当前普通用户的主目录
    if let Some(_tmp) = path.to_str() {
        #[cfg(windows)]
        return _tmp
            .replace(
                "system32\\config\\systemprofile",
                "ServiceProfiles\\LocalService",
            )
            .into();
        #[cfg(target_os = "macos")]
        return _tmp.replace("Application Support", "Preferences").into();
        #[cfg(target_os = "linux")]
        {
            if _tmp == "/root" {
                if let Ok(user) = crate::platform::linux::run_cmds_trim_newline("whoami") {
                    if user != "root" {
                        let cmd = format!("getent passwd '{}' | awk -F':' '{{print $6}}'", user);
                        if let Ok(output) = crate::platform::linux::run_cmds_trim_newline(&cmd) {
                            return output.into();
                        }
                        return format!("/home/{user}").into();
                    }
                }
            }
        }
    }
    path
}

//🧩 5. Config2 的加载、保存与访问接口
//✅ 作用：提供了 Config2（补充配置，如代理、NAT 类型、解锁 PIN、功能选项等）的：
​​//加载（load）​​：从磁盘读取，同时解密敏感字段
​​//保存（store）​​：加密敏感字段后存盘
​​//单例访问​​：通过 CONFIG2（RwLock）实现全局共享、线程安全访问
impl Config2 {
    fn load() -> Config2 {
        /* 加载并解密敏感字段，如 socks密码、unlock_pin */
        let mut config = Config::load_::<Config2>("2");
        let mut store = false;
        if let Some(mut socks) = config.socks {
            let (password, _, store2) =
                decrypt_str_or_original(&socks.password, PASSWORD_ENC_VERSION);
            socks.password = password;
            config.socks = Some(socks);
            store |= store2;
        }
        let (unlock_pin, _, store2) =
            decrypt_str_or_original(&config.unlock_pin, PASSWORD_ENC_VERSION);
        config.unlock_pin = unlock_pin;
        store |= store2;
        if store {
            config.store();
        }
        config
    }

    pub fn file() -> PathBuf {
        /* 返回配置文件路径 */ 
        Config::file_("2")
    }

    fn store(&self) {
        /* 加密敏感字段并保存 */ 
        let mut config = self.clone();
        if let Some(mut socks) = config.socks {
            socks.password =
                encrypt_str_or_original(&socks.password, PASSWORD_ENC_VERSION, ENCRYPT_MAX_LEN);
            config.socks = Some(socks);
        }
        config.unlock_pin =
            encrypt_str_or_original(&config.unlock_pin, PASSWORD_ENC_VERSION, ENCRYPT_MAX_LEN);
        Config::store_(&config, "2");
    }

    pub fn get() -> Config2 {
        /* 读取全局共享的 Config2（线程安全）*/
        return CONFIG2.read().unwrap().clone();
    }

    pub fn set(cfg: Config2) -> bool {
        /* 更新全局 Config2 并持久化 */
        let mut lock = CONFIG2.write().unwrap();
        if *lock == cfg {
            return false;
        }
        *lock = cfg;
        lock.store();
        true
    }
}

//🧩 6. 通用配置加载与存储函数
//✅ 作用：封装了基于 confy的​​通用配置读写逻辑​​，用于所有 Config/ Config2/ 其他结构体，支持：
//自动序列化 / 反序列化
//文件不存在时返回默认值
//错误日志记录
//Unix 文件权限控制（仅限非 Windows）

pub fn load_path<T: serde::Serialize + serde::de::DeserializeOwned + Default + std::fmt::Debug>(
    file: PathBuf,
) -> T {
    /* 基于 confy 库从文件加载任意配置结构体，出错时返回默认值 */
    let cfg = match confy::load_path(&file) {
        Ok(config) => config,
        Err(err) => {
            if let confy::ConfyError::GeneralLoadError(err) = &err {
                if err.kind() == std::io::ErrorKind::NotFound {
                    return T::default();
                }
            }
            log::error!("Failed to load config '{}': {}", file.display(), err);
            T::default()
        }
    };
    cfg
}

#[inline]
pub fn store_path<T: serde::Serialize>(path: PathBuf, cfg: T) -> crate::ResultType<()> {
    /* 基于 confy 保存配置，Unix 下设置 0600 权限 */
    #[cfg(not(windows))]
    {
        use std::os::unix::fs::PermissionsExt;
        Ok(confy::store_path_perms(
            path,
            cfg,
            fs::Permissions::from_mode(0o600),
        )?)
    }
    #[cfg(windows)]
    {
        Ok(confy::store_path(path, cfg)?)
    }
}

//🧩 7. Config 的加载与存储（含 ID 生成与加密逻辑）
//✅ 作用：Config是最核心的配置结构体之一，负责：
//设备唯一标识符（ID）的生成与持久化
//密码、密钥对、加密字段的解密 / 加密
//兼容性处理（比如老版本没有 enc_id 的情况）
//设备首次启动时生成合法 ID（循环尝试直到成功）

impl Config {
    fn load_<T: serde::Serialize + serde::de::DeserializeOwned + Default + std::fmt::Debug>(
        suffix: &str,
    ) -> T {
        /* 加载任意配置结构体（模板方法）*/
        let file = Self::file_(suffix);
        let cfg = load_path(file);
        if suffix.is_empty() {
            log::trace!("{:?}", cfg);
        }
        cfg
    }

    fn store_<T: serde::Serialize>(config: &T, suffix: &str) {
        /* 存储任意配置结构体 */
        let file = Self::file_(suffix);
        if let Err(err) = store_path(file, config) {
            log::error!("Failed to store {suffix} config: {err}");
        }
    }

    fn load() -> Config {
        /* 加载 Config，解密字段如 password, enc_id，必要时生成新设备 ID */
        let mut config = Config::load_::<Config>("");
        let mut store = false;
        let (password, _, store1) = decrypt_str_or_original(&config.password, PASSWORD_ENC_VERSION);
        config.password = password;
        store |= store1;
        let mut id_valid = false;
        let (id, encrypted, store2) = decrypt_str_or_original(&config.enc_id, PASSWORD_ENC_VERSION);
        if encrypted {
            config.id = id;
            id_valid = true;
            store |= store2;
        } else if
        // Comment out for forward compatible
        // crate::get_modified_time(&Self::file_(""))
        // .checked_sub(std::time::Duration::from_secs(30)) // allow modification during installation
        // .unwrap_or_else(crate::get_exe_time)
        // < crate::get_exe_time()
        // &&
        !config.id.is_empty()
            && config.enc_id.is_empty()
            && !decrypt_str_or_original(&config.id, PASSWORD_ENC_VERSION).1
        {
            id_valid = true;
            store = true;
        }
        if !id_valid {
            for _ in 0..3 {
                if let Some(id) = Config::gen_id() {
                    config.id = id;
                    store = true;
                    break;
                } else {
                    log::error!("Failed to generate new id");
                }
            }
        }
        if store {
            config.store();
        }
        config
    }

    fn store(&self) {
        let mut config = self.clone();
        config.password =
            encrypt_str_or_original(&config.password, PASSWORD_ENC_VERSION, ENCRYPT_MAX_LEN);
        config.enc_id = encrypt_str_or_original(&config.id, PASSWORD_ENC_VERSION, ENCRYPT_MAX_LEN);
        config.id = "".to_owned();
        Config::store_(&config, "");
    }

    pub fn file() -> PathBuf {
        Self::file_("")
    }

    fn file_(suffix: &str) -> PathBuf {
        let name = format!("{}{}", *APP_NAME.read().unwrap(), suffix);
        Config::with_extension(Self::path(name))
    }

    pub fn is_empty(&self) -> bool {
        (self.id.is_empty() && self.enc_id.is_empty()) || self.key_pair.0.is_empty()
    }

    pub fn get_home() -> PathBuf {
        #[cfg(any(target_os = "android", target_os = "ios"))]
        return PathBuf::from(APP_HOME_DIR.read().unwrap().as_str());
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            if let Some(path) = dirs_next::home_dir() {
                patch(path)
            } else if let Ok(path) = std::env::current_dir() {
                path
            } else {
                std::env::temp_dir()
            }
        }
    }

    pub fn path<P: AsRef<Path>>(p: P) -> PathBuf {
        #[cfg(any(target_os = "android", target_os = "ios"))]
        {
            let mut path: PathBuf = APP_DIR.read().unwrap().clone().into();
            path.push(p);
            return path;
        }
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            #[cfg(not(target_os = "macos"))]
            let org = "".to_owned();
            #[cfg(target_os = "macos")]
            let org = ORG.read().unwrap().clone();
            // /var/root for root
            if let Some(project) =
                directories_next::ProjectDirs::from("", &org, &APP_NAME.read().unwrap())
            {
                let mut path = patch(project.config_dir().to_path_buf());
                path.push(p);
                return path;
            }
            "".into()
        }
    }

    #[allow(unreachable_code)]
    pub fn log_path() -> PathBuf {
        #[cfg(target_os = "macos")]
        {
            if let Some(path) = dirs_next::home_dir().as_mut() {
                path.push(format!("Library/Logs/{}", *APP_NAME.read().unwrap()));
                return path.clone();
            }
        }
        #[cfg(target_os = "linux")]
        {
            let mut path = Self::get_home();
            path.push(format!(".local/share/logs/{}", *APP_NAME.read().unwrap()));
            std::fs::create_dir_all(&path).ok();
            return path;
        }
        #[cfg(target_os = "android")]
        {
            let mut path = Self::get_home();
            path.push(format!("{}/Logs", *APP_NAME.read().unwrap()));
            std::fs::create_dir_all(&path).ok();
            return path;
        }
        if let Some(path) = Self::path("").parent() {
            let mut path: PathBuf = path.into();
            path.push("log");
            return path;
        }
        "".into()
    }

    pub fn ipc_path(postfix: &str) -> String {
        #[cfg(windows)]
        {
            // \\ServerName\pipe\PipeName
            // where ServerName is either the name of a remote computer or a period, to specify the local computer.
            // https://docs.microsoft.com/en-us/windows/win32/ipc/pipe-names
            format!(
                "\\\\.\\pipe\\{}\\query{}",
                *APP_NAME.read().unwrap(),
                postfix
            )
        }
        #[cfg(not(windows))]
        {
            use std::os::unix::fs::PermissionsExt;
            #[cfg(target_os = "android")]
            let mut path: PathBuf =
                format!("{}/{}", *APP_DIR.read().unwrap(), *APP_NAME.read().unwrap()).into();
            #[cfg(not(target_os = "android"))]
            let mut path: PathBuf = format!("/tmp/{}", *APP_NAME.read().unwrap()).into();
            fs::create_dir(&path).ok();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o0777)).ok();
            path.push(format!("ipc{postfix}"));
            path.to_str().unwrap_or("").to_owned()
        }
    }

    pub fn icon_path() -> PathBuf {
        let mut path = Self::path("icons");
        if fs::create_dir_all(&path).is_err() {
            path = std::env::temp_dir();
        }
        path
    }

    #[inline]
    pub fn get_any_listen_addr(is_ipv4: bool) -> SocketAddr {
        if is_ipv4 {
            SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)
        } else {
            SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 0)
        }
    }

    pub fn get_rendezvous_server() -> String {
        let mut rendezvous_server = EXE_RENDEZVOUS_SERVER.read().unwrap().clone();
        if rendezvous_server.is_empty() {
            rendezvous_server = Self::get_option("custom-rendezvous-server");
        }
        if rendezvous_server.is_empty() {
            rendezvous_server = PROD_RENDEZVOUS_SERVER.read().unwrap().clone();
        }
        if rendezvous_server.is_empty() {
            rendezvous_server = CONFIG2.read().unwrap().rendezvous_server.clone();
        }
        if rendezvous_server.is_empty() {
            rendezvous_server = Self::get_rendezvous_servers()
                .drain(..)
                .next()
                .unwrap_or_default();
        }
        if !rendezvous_server.contains(':') {
            rendezvous_server = format!("{rendezvous_server}:{RENDEZVOUS_PORT}");
        }
        rendezvous_server
    }

    pub fn get_rendezvous_servers() -> Vec<String> {
        let s = EXE_RENDEZVOUS_SERVER.read().unwrap().clone();
        if !s.is_empty() {
            return vec![s];
        }
        let s = Self::get_option("custom-rendezvous-server");
        if !s.is_empty() {
            return vec![s];
        }
        let s = PROD_RENDEZVOUS_SERVER.read().unwrap().clone();
        if !s.is_empty() {
            return vec![s];
        }
        let serial_obsolute = CONFIG2.read().unwrap().serial > SERIAL;
        if serial_obsolute {
            let ss: Vec<String> = Self::get_option("rendezvous-servers")
                .split(',')
                .filter(|x| x.contains('.'))
                .map(|x| x.to_owned())
                .collect();
            if !ss.is_empty() {
                return ss;
            }
        }
        return RENDEZVOUS_SERVERS.iter().map(|x| x.to_string()).collect();
    }

    pub fn reset_online() {
        *ONLINE.lock().unwrap() = Default::default();
    }

    pub fn update_latency(host: &str, latency: i64) {
        ONLINE.lock().unwrap().insert(host.to_owned(), latency);
        let mut host = "".to_owned();
        let mut delay = i64::MAX;
        for (tmp_host, tmp_delay) in ONLINE.lock().unwrap().iter() {
            if tmp_delay > &0 && tmp_delay < &delay {
                delay = *tmp_delay;
                host = tmp_host.to_string();
            }
        }
        if !host.is_empty() {
            let mut config = CONFIG2.write().unwrap();
            if host != config.rendezvous_server {
                log::debug!("Update rendezvous_server in config to {}", host);
                log::debug!("{:?}", *ONLINE.lock().unwrap());
                config.rendezvous_server = host;
                config.store();
            }
        }
    }

    pub fn set_id(id: &str) {
        let mut config = CONFIG.write().unwrap();
        if id == config.id {
            return;
        }
        config.id = id.into();
        config.store();
    }

    pub fn set_nat_type(nat_type: i32) {
        let mut config = CONFIG2.write().unwrap();
        if nat_type == config.nat_type {
            return;
        }
        config.nat_type = nat_type;
        config.store();
    }

    pub fn get_nat_type() -> i32 {
        CONFIG2.read().unwrap().nat_type
    }

    pub fn set_serial(serial: i32) {
        let mut config = CONFIG2.write().unwrap();
        if serial == config.serial {
            return;
        }
        config.serial = serial;
        config.store();
    }

    pub fn get_serial() -> i32 {
        std::cmp::max(CONFIG2.read().unwrap().serial, SERIAL)
    }

    #[cfg(any(target_os = "android", target_os = "ios"))]
    fn gen_id() -> Option<String> {
        Self::get_auto_id()
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    fn gen_id() -> Option<String> {
        let hostname_as_id = BUILTIN_SETTINGS
            .read()
            .unwrap()
            .get(keys::OPTION_ALLOW_HOSTNAME_AS_ID)
            .map(|v| option2bool(keys::OPTION_ALLOW_HOSTNAME_AS_ID, v))
            .unwrap_or(false);
        if hostname_as_id {
            match whoami::fallible::hostname() {
                Ok(h) => Some(h.replace(" ", "-")),
                Err(e) => {
                    log::warn!("Failed to get hostname, \"{}\", fallback to auto id", e);
                    Self::get_auto_id()
                }
            }
        } else {
            Self::get_auto_id()
        }
    }

    fn get_auto_id() -> Option<String> {
        #[cfg(any(target_os = "android", target_os = "ios"))]
        {
            return Some(
                rand::thread_rng()
                    .gen_range(1_000_000_000..2_000_000_000)
                    .to_string(),
            );
        }

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            let mut id = 0u32;
            if let Ok(Some(ma)) = mac_address::get_mac_address() {
                for x in &ma.bytes()[2..] {
                    id = (id << 8) | (*x as u32);
                }
                id &= 0x1FFFFFFF;
                Some(id.to_string())
            } else {
                None
            }
        }
    }

    pub fn get_auto_password(length: usize) -> String {
        Self::get_auto_password_with_chars(length, CHARS)
    }

    pub fn get_auto_numeric_password(length: usize) -> String {
        Self::get_auto_password_with_chars(length, NUM_CHARS)
    }

    fn get_auto_password_with_chars(length: usize, chars: &[char]) -> String {
        let mut rng = rand::thread_rng();
        (0..length)
            .map(|_| chars[rng.gen::<usize>() % chars.len()])
            .collect()
    }

    pub fn get_key_confirmed() -> bool {
        CONFIG.read().unwrap().key_confirmed
    }

    pub fn set_key_confirmed(v: bool) {
        let mut config = CONFIG.write().unwrap();
        if config.key_confirmed == v {
            return;
        }
        config.key_confirmed = v;
        if !v {
            config.keys_confirmed = Default::default();
        }
        config.store();
    }

    pub fn get_host_key_confirmed(host: &str) -> bool {
        matches!(CONFIG.read().unwrap().keys_confirmed.get(host), Some(true))
    }

    pub fn set_host_key_confirmed(host: &str, v: bool) {
        if Self::get_host_key_confirmed(host) == v {
            return;
        }
        let mut config = CONFIG.write().unwrap();
        config.keys_confirmed.insert(host.to_owned(), v);
        config.store();
    }

    pub fn get_key_pair() -> KeyPair {
        // lock here to make sure no gen_keypair more than once
        // no use of CONFIG directly here to ensure no recursive calling in Config::load because of password dec which calling this function
        let mut lock = KEY_PAIR.lock().unwrap();
        if let Some(p) = lock.as_ref() {
            return p.clone();
        }
        let mut config = Config::load_::<Config>("");
        if config.key_pair.0.is_empty() {
            log::info!("Generated new keypair for id: {}", config.id);
            let (pk, sk) = sign::gen_keypair();
            let key_pair = (sk.0.to_vec(), pk.0.into());
            config.key_pair = key_pair.clone();
            std::thread::spawn(|| {
                let mut config = CONFIG.write().unwrap();
                config.key_pair = key_pair;
                config.store();
            });
        }
        *lock = Some(config.key_pair.clone());
        config.key_pair
    }

    pub fn no_register_device() -> bool {
        BUILTIN_SETTINGS
            .read()
            .unwrap()
            .get(keys::OPTION_REGISTER_DEVICE)
            .map(|v| v == "N")
            .unwrap_or(false)
    }

    pub fn get_id() -> String {
        let mut id = CONFIG.read().unwrap().id.clone();
        if id.is_empty() {
            if let Some(tmp) = Config::gen_id() {
                id = tmp;
                Config::set_id(&id);
            }
        }
        id
    }

    pub fn get_id_or(b: String) -> String {
        let a = CONFIG.read().unwrap().id.clone();
        if a.is_empty() {
            b
        } else {
            a
        }
    }

    pub fn get_options() -> HashMap<String, String> {
        let mut res = DEFAULT_SETTINGS.read().unwrap().clone();
        res.extend(CONFIG2.read().unwrap().options.clone());
        res.extend(OVERWRITE_SETTINGS.read().unwrap().clone());
        res
    }

    #[inline]
    fn purify_options(v: &mut HashMap<String, String>) {
        v.retain(|k, v| is_option_can_save(&OVERWRITE_SETTINGS, k, &DEFAULT_SETTINGS, v));
    }

    pub fn set_options(mut v: HashMap<String, String>) {
        Self::purify_options(&mut v);
        let mut config = CONFIG2.write().unwrap();
        if config.options == v {
            return;
        }
        config.options = v;
        config.store();
    }

    pub fn get_option(k: &str) -> String {
        get_or(
            &OVERWRITE_SETTINGS,
            &CONFIG2.read().unwrap().options,
            &DEFAULT_SETTINGS,
            k,
        )
        .unwrap_or_default()
    }

    pub fn get_bool_option(k: &str) -> bool {
        option2bool(k, &Self::get_option(k))
    }

    pub fn set_option(k: String, v: String) {
        if !is_option_can_save(&OVERWRITE_SETTINGS, &k, &DEFAULT_SETTINGS, &v) {
            return;
        }
        let mut config = CONFIG2.write().unwrap();
        let v2 = if v.is_empty() { None } else { Some(&v) };
        if v2 != config.options.get(&k) {
            if v2.is_none() {
                config.options.remove(&k);
            } else {
                config.options.insert(k, v);
            }
            config.store();
        }
    }

    pub fn update_id() {
        // to-do: how about if one ip register a lot of ids?
        let id = Self::get_id();
        let mut rng = rand::thread_rng();
        let new_id = rng.gen_range(1_000_000_000..2_000_000_000).to_string();
        Config::set_id(&new_id);
        log::info!("id updated from {} to {}", id, new_id);
    }

    pub fn set_permanent_password(password: &str) {
        if HARD_SETTINGS
            .read()
            .unwrap()
            .get("password")
            .map_or(false, |v| v == password)
        {
            return;
        }
        let mut config = CONFIG.write().unwrap();
        if password == config.password {
            return;
        }
        config.password = password.into();
        config.store();
        Self::clear_trusted_devices();
    }

    pub fn get_permanent_password() -> String {
        let mut password = CONFIG.read().unwrap().password.clone();
        if password.is_empty() {
            if let Some(v) = HARD_SETTINGS.read().unwrap().get("password") {
                password = v.to_owned();
            }
        }
        password
    }

    pub fn set_salt(salt: &str) {
        let mut config = CONFIG.write().unwrap();
        if salt == config.salt {
            return;
        }
        config.salt = salt.into();
        config.store();
    }

    pub fn get_salt() -> String {
        let mut salt = CONFIG.read().unwrap().salt.clone();
        if salt.is_empty() {
            salt = Config::get_auto_password(6);
            Config::set_salt(&salt);
        }
        salt
    }

    pub fn set_socks(socks: Option<Socks5Server>) {
        if OVERWRITE_SETTINGS
            .read()
            .unwrap()
            .contains_key(keys::OPTION_PROXY_URL)
        {
            return;
        }

        let mut config = CONFIG2.write().unwrap();
        if config.socks == socks {
            return;
        }
        if config.socks.is_none() {
            let equal_to_default = |key: &str, value: &str| {
                DEFAULT_SETTINGS
                    .read()
                    .unwrap()
                    .get(key)
                    .map_or(false, |x| *x == value)
            };
            let contains_url = DEFAULT_SETTINGS
                .read()
                .unwrap()
                .get(keys::OPTION_PROXY_URL)
                .is_some();
            let url = equal_to_default(
                keys::OPTION_PROXY_URL,
                &socks.clone().unwrap_or_default().proxy,
            );
            let username = equal_to_default(
                keys::OPTION_PROXY_USERNAME,
                &socks.clone().unwrap_or_default().username,
            );
            let password = equal_to_default(
                keys::OPTION_PROXY_PASSWORD,
                &socks.clone().unwrap_or_default().password,
            );
            if contains_url && url && username && password {
                return;
            }
        }
        config.socks = socks;
        config.store();
    }

    #[inline]
    fn get_socks_from_custom_client_advanced_settings(
        settings: &HashMap<String, String>,
    ) -> Option<Socks5Server> {
        let url = settings.get(keys::OPTION_PROXY_URL)?;
        Some(Socks5Server {
            proxy: url.to_owned(),
            username: settings
                .get(keys::OPTION_PROXY_USERNAME)
                .map(|x| x.to_string())
                .unwrap_or_default(),
            password: settings
                .get(keys::OPTION_PROXY_PASSWORD)
                .map(|x| x.to_string())
                .unwrap_or_default(),
        })
    }

    pub fn get_socks() -> Option<Socks5Server> {
        Self::get_socks_from_custom_client_advanced_settings(&OVERWRITE_SETTINGS.read().unwrap())
            .or(CONFIG2.read().unwrap().socks.clone())
            .or(Self::get_socks_from_custom_client_advanced_settings(
                &DEFAULT_SETTINGS.read().unwrap(),
            ))
    }

    #[inline]
    pub fn is_proxy() -> bool {
        Self::get_network_type() != NetworkType::Direct
    }

    pub fn get_network_type() -> NetworkType {
        if OVERWRITE_SETTINGS
            .read()
            .unwrap()
            .get(keys::OPTION_PROXY_URL)
            .is_some()
        {
            return NetworkType::ProxySocks;
        }
        if CONFIG2.read().unwrap().socks.is_some() {
            return NetworkType::ProxySocks;
        }
        if DEFAULT_SETTINGS
            .read()
            .unwrap()
            .get(keys::OPTION_PROXY_URL)
            .is_some()
        {
            return NetworkType::ProxySocks;
        }
        NetworkType::Direct
    }

    pub fn get_unlock_pin() -> String {
        CONFIG2.read().unwrap().unlock_pin.clone()
    }

    pub fn set_unlock_pin(pin: &str) {
        let mut config = CONFIG2.write().unwrap();
        if pin == config.unlock_pin {
            return;
        }
        config.unlock_pin = pin.to_string();
        config.store();
    }

    pub fn get_trusted_devices_json() -> String {
        serde_json::to_string(&Self::get_trusted_devices()).unwrap_or_default()
    }

    pub fn get_trusted_devices() -> Vec<TrustedDevice> {
        let (devices, synced) = TRUSTED_DEVICES.read().unwrap().clone();
        if synced {
            return devices;
        }
        let devices = CONFIG2.read().unwrap().trusted_devices.clone();
        let (devices, succ, store) = decrypt_str_or_original(&devices, PASSWORD_ENC_VERSION);
        if succ {
            let mut devices: Vec<TrustedDevice> =
                serde_json::from_str(&devices).unwrap_or_default();
            let len = devices.len();
            devices.retain(|d| !d.outdate());
            if store || devices.len() != len {
                Self::set_trusted_devices(devices.clone());
            }
            *TRUSTED_DEVICES.write().unwrap() = (devices.clone(), true);
            devices
        } else {
            Default::default()
        }
    }

    fn set_trusted_devices(mut trusted_devices: Vec<TrustedDevice>) {
        trusted_devices.retain(|d| !d.outdate());
        let devices = serde_json::to_string(&trusted_devices).unwrap_or_default();
        let max_len = 1024 * 1024;
        if devices.bytes().len() > max_len {
            log::error!("Trusted devices too large: {}", devices.bytes().len());
            return;
        }
        let devices = encrypt_str_or_original(&devices, PASSWORD_ENC_VERSION, max_len);
        let mut config = CONFIG2.write().unwrap();
        config.trusted_devices = devices;
        config.store();
        *TRUSTED_DEVICES.write().unwrap() = (trusted_devices, true);
    }

    pub fn add_trusted_device(device: TrustedDevice) {
        let mut devices = Self::get_trusted_devices();
        devices.retain(|d| d.hwid != device.hwid);
        devices.push(device);
        Self::set_trusted_devices(devices);
    }

    pub fn remove_trusted_devices(hwids: &Vec<Bytes>) {
        let mut devices = Self::get_trusted_devices();
        devices.retain(|d| !hwids.contains(&d.hwid));
        Self::set_trusted_devices(devices);
    }

    pub fn clear_trusted_devices() {
        Self::set_trusted_devices(Default::default());
    }

    pub fn get() -> Config {
        return CONFIG.read().unwrap().clone();
    }

    pub fn set(cfg: Config) -> bool {
        let mut lock = CONFIG.write().unwrap();
        if *lock == cfg {
            return false;
        }
        *lock = cfg;
        lock.store();
        true
    }

    fn with_extension(path: PathBuf) -> PathBuf {
        let ext = path.extension();
        if let Some(ext) = ext {
            let ext = format!("{}.toml", ext.to_string_lossy());
            path.with_extension(ext)
        } else {
            path.with_extension("toml")
        }
    }
}

const PEERS: &str = "peers";

impl PeerConfig {
    pub fn load(id: &str) -> PeerConfig {
        let _lock = CONFIG.read().unwrap();
        match confy::load_path(Self::path(id)) {
            Ok(config) => {
                let mut config: PeerConfig = config;
                let mut store = false;
                let (password, _, store2) =
                    decrypt_vec_or_original(&config.password, PASSWORD_ENC_VERSION);
                config.password = password;
                store = store || store2;
                for opt in ["rdp_password", "os-username", "os-password"] {
                    if let Some(v) = config.options.get_mut(opt) {
                        let (encrypted, _, store2) =
                            decrypt_str_or_original(v, PASSWORD_ENC_VERSION);
                        *v = encrypted;
                        store = store || store2;
                    }
                }
                if store {
                    config.store_(id);
                }
                config
            }
            Err(err) => {
                if let confy::ConfyError::GeneralLoadError(err) = &err {
                    if err.kind() == std::io::ErrorKind::NotFound {
                        return Default::default();
                    }
                }
                log::error!("Failed to load peer config '{}': {}", id, err);
                Default::default()
            }
        }
    }

    pub fn store(&self, id: &str) {
        let _lock = CONFIG.read().unwrap();
        self.store_(id);
    }

    fn store_(&self, id: &str) {
        let mut config = self.clone();
        config.password =
            encrypt_vec_or_original(&config.password, PASSWORD_ENC_VERSION, ENCRYPT_MAX_LEN);
        for opt in ["rdp_password", "os-username", "os-password"] {
            if let Some(v) = config.options.get_mut(opt) {
                *v = encrypt_str_or_original(v, PASSWORD_ENC_VERSION, ENCRYPT_MAX_LEN)
            }
        }
        if let Err(err) = store_path(Self::path(id), config) {
            log::error!("Failed to store config: {}", err);
        }
        NEW_STORED_PEER_CONFIG.lock().unwrap().insert(id.to_owned());
    }

    pub fn remove(id: &str) {
        fs::remove_file(Self::path(id)).ok();
    }

    fn path(id: &str) -> PathBuf {
        //If the id contains invalid chars, encode it
        let forbidden_paths = Regex::new(r".*[<>:/\\|\?\*].*");
        let path: PathBuf;
        if let Ok(forbidden_paths) = forbidden_paths {
            let id_encoded = if forbidden_paths.is_match(id) {
                "base64_".to_string() + base64::encode(id, base64::Variant::Original).as_str()
            } else {
                id.to_string()
            };
            path = [PEERS, id_encoded.as_str()].iter().collect();
        } else {
            log::warn!("Regex create failed: {:?}", forbidden_paths.err());
            // fallback for failing to create this regex.
            path = [PEERS, id.replace(":", "_").as_str()].iter().collect();
        }
        Config::with_extension(Config::path(path))
    }

    // The number of peers to load in the first round when showing the peers card list in the main window.
    // When there're too many peers, loading all of them at once will take a long time.
    // We can load them in two rouds, the first round loads the first 100 peers, and the second round loads the rest.
    // Then the UI will show the first 100 peers first, and the rest will be loaded and shown later.
    pub const BATCH_LOADING_COUNT: usize = 100;

    pub fn get_vec_id_modified_time_path(
        id_filters: &Option<Vec<String>>,
    ) -> Vec<(String, SystemTime, PathBuf)> {
        if let Ok(peers) = Config::path(PEERS).read_dir() {
            let mut vec_id_modified_time_path = peers
                .into_iter()
                .filter_map(|res| match res {
                    Ok(res) => {
                        let p = res.path();
                        if p.is_file()
                            && p.extension().map(|p| p.to_str().unwrap_or("")) == Some("toml")
                        {
                            Some(p)
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
                .map(|p| {
                    let id = p
                        .file_stem()
                        .map(|p| p.to_str().unwrap_or(""))
                        .unwrap_or("")
                        .to_owned();

                    let id_decoded_string = if id.starts_with("base64_") && id.len() != 7 {
                        let id_decoded =
                            base64::decode(&id[7..], base64::Variant::Original).unwrap_or_default();
                        String::from_utf8_lossy(&id_decoded).as_ref().to_owned()
                    } else {
                        id
                    };
                    (id_decoded_string, p)
                })
                .filter(|(id, _)| {
                    let Some(filters) = id_filters else {
                        return true;
                    };
                    filters.contains(id)
                })
                .map(|(id, p)| {
                    let t = crate::get_modified_time(&p);
                    (id, t, p)
                })
                .collect::<Vec<_>>();
            vec_id_modified_time_path.sort_unstable_by(|a, b| b.1.cmp(&a.1));
            vec_id_modified_time_path
        } else {
            vec![]
        }
    }

    #[inline]
    async fn preload_file_async(path: PathBuf) {
        let _ = tokio::fs::File::open(path).await;
    }

    #[tokio::main(flavor = "current_thread")]
    async fn preload_peers_async() {
        let now = std::time::Instant::now();
        let vec_id_modified_time_path = Self::get_vec_id_modified_time_path(&None);
        let total_count = vec_id_modified_time_path.len();
        let mut futs = vec![];
        for (_, _, path) in vec_id_modified_time_path.into_iter() {
            futs.push(Self::preload_file_async(path));
            if futs.len() >= Self::BATCH_LOADING_COUNT {
                let first_load_start = std::time::Instant::now();
                futures::future::join_all(futs).await;
                if first_load_start.elapsed().as_millis() < 10 {
                    // No need to preload the rest if the first load is fast.
                    return;
                }
                futs = vec![];
            }
        }
        if !futs.is_empty() {
            futures::future::join_all(futs).await;
        }
        log::info!(
            "Preload peers done in {:?}, batch_count: {}, total: {}",
            now.elapsed(),
            Self::BATCH_LOADING_COUNT,
            total_count
        );
    }

    // We have to preload all peers in a background thread.
    // Because we find that opening files the first time after the system (Windows) booting will be very slow, up to 200~400ms.
    // The reason is that the Windows has "Microsoft Defender Antivirus Service" running in the background, which will scan the file when it's opened the first time.
    // So we have to preload all peers in a background thread to avoid the delay when opening the file the first time.
    // We can temporarily stop "Microsoft Defender Antivirus Service" or add the fold to the white list, to verify this. But don't do this in the release version.
    pub fn preload_peers() {
        std::thread::spawn(|| {
            Self::preload_peers_async();
        });
    }

    pub fn peers(id_filters: Option<Vec<String>>) -> Vec<(String, SystemTime, PeerConfig)> {
        let vec_id_modified_time_path = Self::get_vec_id_modified_time_path(&id_filters);
        Self::batch_peers(
            &vec_id_modified_time_path,
            0,
            Some(vec_id_modified_time_path.len()),
        )
        .0
    }

    pub fn batch_peers(
        all: &Vec<(String, SystemTime, PathBuf)>,
        from: usize,
        to: Option<usize>,
    ) -> (Vec<(String, SystemTime, PeerConfig)>, usize) {
        if from >= all.len() {
            return (vec![], 0);
        }

        let to = match to {
            Some(to) => to.min(all.len()),
            None => (from + Self::BATCH_LOADING_COUNT).min(all.len()),
        };

        // to <= from is unexpected, but we can just return an empty vec in this case.
        if to <= from {
            return (vec![], from);
        }

        let peers: Vec<_> = all[from..to]
            .iter()
            .map(|(id, t, p)| {
                let c = PeerConfig::load(&id);
                if c.info.platform.is_empty() {
                    fs::remove_file(p).ok();
                }
                (id.clone(), t.clone(), c)
            })
            .filter(|p| !p.2.info.platform.is_empty())
            .collect();
        (peers, to)
    }

    pub fn exists(id: &str) -> bool {
        Self::path(id).exists()
    }

    serde_field_string!(
        default_view_style,
        deserialize_view_style,
        UserDefaultConfig::read(keys::OPTION_VIEW_STYLE)
    );
    serde_field_string!(
        default_scroll_style,
        deserialize_scroll_style,
        UserDefaultConfig::read(keys::OPTION_SCROLL_STYLE)
    );
    serde_field_string!(
        default_image_quality,
        deserialize_image_quality,
        UserDefaultConfig::read(keys::OPTION_IMAGE_QUALITY)
    );
    serde_field_string!(
        default_reverse_mouse_wheel,
        deserialize_reverse_mouse_wheel,
        UserDefaultConfig::read(keys::OPTION_REVERSE_MOUSE_WHEEL)
    );
    serde_field_string!(
        default_displays_as_individual_windows,
        deserialize_displays_as_individual_windows,
        UserDefaultConfig::read(keys::OPTION_DISPLAYS_AS_INDIVIDUAL_WINDOWS)
    );
    serde_field_string!(
        default_use_all_my_displays_for_the_remote_session,
        deserialize_use_all_my_displays_for_the_remote_session,
        UserDefaultConfig::read(keys::OPTION_USE_ALL_MY_DISPLAYS_FOR_THE_REMOTE_SESSION)
    );

    fn default_custom_image_quality() -> Vec<i32> {
        let f: f64 = UserDefaultConfig::read(keys::OPTION_CUSTOM_IMAGE_QUALITY)
            .parse()
            .unwrap_or(50.0);
        vec![f as _]
    }

    fn deserialize_custom_image_quality<'de, D>(deserializer: D) -> Result<Vec<i32>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let v: Vec<i32> = de::Deserialize::deserialize(deserializer)?;
        if v.len() == 1 && v[0] >= 10 && v[0] <= 0xFFF {
            Ok(v)
        } else {
            Ok(Self::default_custom_image_quality())
        }
    }

    fn default_options() -> HashMap<String, String> {
        let mut mp: HashMap<String, String> = Default::default();
        [
            keys::OPTION_CODEC_PREFERENCE,
            keys::OPTION_CUSTOM_FPS,
            keys::OPTION_ZOOM_CURSOR,
            keys::OPTION_TOUCH_MODE,
            keys::OPTION_I444,
            keys::OPTION_SWAP_LEFT_RIGHT_MOUSE,
            keys::OPTION_COLLAPSE_TOOLBAR,
        ]
        .map(|key| {
            mp.insert(key.to_owned(), UserDefaultConfig::read(key));
        });
        mp
    }

    fn default_trackpad_speed() -> i32 {
        UserDefaultConfig::read(keys::OPTION_TRACKPAD_SPEED)
            .parse()
            .unwrap_or(100)
    }

    fn deserialize_trackpad_speed<'de, D>(deserializer: D) -> Result<i32, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let v: i32 = de::Deserialize::deserialize(deserializer)?;
        if v >= 10 && v <= 1000 {
            Ok(v)
        } else {
            Ok(Self::default_trackpad_speed())
        }
    }
}

serde_field_bool!(
    ShowRemoteCursor,
    "show_remote_cursor",
    default_show_remote_cursor,
    "ShowRemoteCursor::default_show_remote_cursor"
);
serde_field_bool!(
    FollowRemoteCursor,
    "follow_remote_cursor",
    default_follow_remote_cursor,
    "FollowRemoteCursor::default_follow_remote_cursor"
);

serde_field_bool!(
    FollowRemoteWindow,
    "follow_remote_window",
    default_follow_remote_window,
    "FollowRemoteWindow::default_follow_remote_window"
);
serde_field_bool!(
    ShowQualityMonitor,
    "show_quality_monitor",
    default_show_quality_monitor,
    "ShowQualityMonitor::default_show_quality_monitor"
);
serde_field_bool!(
    DisableAudio,
    "disable_audio",
    default_disable_audio,
    "DisableAudio::default_disable_audio"
);
serde_field_bool!(
    EnableFileCopyPaste,
    "enable-file-copy-paste",
    default_enable_file_copy_paste,
    "EnableFileCopyPaste::default_enable_file_copy_paste"
);
serde_field_bool!(
    DisableClipboard,
    "disable_clipboard",
    default_disable_clipboard,
    "DisableClipboard::default_disable_clipboard"
);
serde_field_bool!(
    LockAfterSessionEnd,
    "lock_after_session_end",
    default_lock_after_session_end,
    "LockAfterSessionEnd::default_lock_after_session_end"
);
serde_field_bool!(
    TerminalPersistent,
    "terminal-persistent",
    default_terminal_persistent,
    "TerminalPersistent::default_terminal_persistent"
);
serde_field_bool!(
    PrivacyMode,
    "privacy_mode",
    default_privacy_mode,
    "PrivacyMode::default_privacy_mode"
);

serde_field_bool!(
    AllowSwapKey,
    "allow_swap_key",
    default_allow_swap_key,
    "AllowSwapKey::default_allow_swap_key"
);

serde_field_bool!(
    ViewOnly,
    "view_only",
    default_view_only,
    "ViewOnly::default_view_only"
);

serde_field_bool!(
    ShowMyCursor,
    "show_my_cursor",
    default_show_my_cursor,
    "ShowMyCursor::default_show_my_cursor"
);

serde_field_bool!(
    SyncInitClipboard,
    "sync-init-clipboard",
    default_sync_init_clipboard,
    "SyncInitClipboard::default_sync_init_clipboard"
);

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct LocalConfig {
    #[serde(default, deserialize_with = "deserialize_string")]
    remote_id: String, // latest used one
    #[serde(default, deserialize_with = "deserialize_string")]
    kb_layout_type: String,
    #[serde(default, deserialize_with = "deserialize_size")]
    size: Size,
    #[serde(default, deserialize_with = "deserialize_vec_string")]
    pub fav: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_hashmap_string_string")]
    options: HashMap<String, String>,
    // Various data for flutter ui
    #[serde(default, deserialize_with = "deserialize_hashmap_string_string")]
    ui_flutter: HashMap<String, String>,
}

impl LocalConfig {
    fn load() -> LocalConfig {
        Config::load_::<LocalConfig>("_local")
    }

    fn store(&self) {
        Config::store_(self, "_local");
    }

    pub fn get_kb_layout_type() -> String {
        LOCAL_CONFIG.read().unwrap().kb_layout_type.clone()
    }

    pub fn set_kb_layout_type(kb_layout_type: String) {
        let mut config = LOCAL_CONFIG.write().unwrap();
        config.kb_layout_type = kb_layout_type;
        config.store();
    }

    pub fn get_size() -> Size {
        LOCAL_CONFIG.read().unwrap().size
    }

    pub fn set_size(x: i32, y: i32, w: i32, h: i32) {
        let mut config = LOCAL_CONFIG.write().unwrap();
        let size = (x, y, w, h);
        if size == config.size || size.2 < 300 || size.3 < 300 {
            return;
        }
        config.size = size;
        config.store();
    }

    pub fn set_remote_id(remote_id: &str) {
        let mut config = LOCAL_CONFIG.write().unwrap();
        if remote_id == config.remote_id {
            return;
        }
        config.remote_id = remote_id.into();
        config.store();
    }

    pub fn get_remote_id() -> String {
        LOCAL_CONFIG.read().unwrap().remote_id.clone()
    }

    pub fn set_fav(fav: Vec<String>) {
        let mut lock = LOCAL_CONFIG.write().unwrap();
        if lock.fav == fav {
            return;
        }
        lock.fav = fav;
        lock.store();
    }

    pub fn get_fav() -> Vec<String> {
        LOCAL_CONFIG.read().unwrap().fav.clone()
    }

    pub fn get_option(k: &str) -> String {
        get_or(
            &OVERWRITE_LOCAL_SETTINGS,
            &LOCAL_CONFIG.read().unwrap().options,
            &DEFAULT_LOCAL_SETTINGS,
            k,
        )
        .unwrap_or_default()
    }

    // Usually get_option should be used.
    pub fn get_option_from_file(k: &str) -> String {
        get_or(
            &OVERWRITE_LOCAL_SETTINGS,
            &Self::load().options,
            &DEFAULT_LOCAL_SETTINGS,
            k,
        )
        .unwrap_or_default()
    }

    pub fn get_bool_option(k: &str) -> bool {
        option2bool(k, &Self::get_option(k))
    }

    pub fn set_option(k: String, v: String) {
        if !is_option_can_save(&OVERWRITE_LOCAL_SETTINGS, &k, &DEFAULT_LOCAL_SETTINGS, &v) {
            return;
        }
        let mut config = LOCAL_CONFIG.write().unwrap();
        // The custom client will explictly set "default" as the default language.
        let is_custom_client_default_lang = k == keys::OPTION_LANGUAGE && v == "default";
        if is_custom_client_default_lang {
            config.options.insert(k, "".to_owned());
            config.store();
            return;
        }
        let v2 = if v.is_empty() { None } else { Some(&v) };
        if v2 != config.options.get(&k) {
            if v2.is_none() {
                config.options.remove(&k);
            } else {
                config.options.insert(k, v);
            }
            config.store();
        }
    }

    pub fn get_flutter_option(k: &str) -> String {
        get_or(
            &OVERWRITE_LOCAL_SETTINGS,
            &LOCAL_CONFIG.read().unwrap().ui_flutter,
            &DEFAULT_LOCAL_SETTINGS,
            k,
        )
        .unwrap_or_default()
    }

    pub fn set_flutter_option(k: String, v: String) {
        let mut config = LOCAL_CONFIG.write().unwrap();
        let v2 = if v.is_empty() { None } else { Some(&v) };
        if v2 != config.ui_flutter.get(&k) {
            if v2.is_none() {
                config.ui_flutter.remove(&k);
            } else {
                config.ui_flutter.insert(k, v);
            }
            config.store();
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct DiscoveryPeer {
    #[serde(default, deserialize_with = "deserialize_string")]
    pub id: String,
    #[serde(default, deserialize_with = "deserialize_string")]
    pub username: String,
    #[serde(default, deserialize_with = "deserialize_string")]
    pub hostname: String,
    #[serde(default, deserialize_with = "deserialize_string")]
    pub platform: String,
    #[serde(default, deserialize_with = "deserialize_bool")]
    pub online: bool,
    #[serde(default, deserialize_with = "deserialize_hashmap_string_string")]
    pub ip_mac: HashMap<String, String>,
}

impl DiscoveryPeer {
    pub fn is_same_peer(&self, other: &DiscoveryPeer) -> bool {
        self.id == other.id && self.username == other.username
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct LanPeers {
    #[serde(default, deserialize_with = "deserialize_vec_discoverypeer")]
    pub peers: Vec<DiscoveryPeer>,
}

impl LanPeers {
    pub fn load() -> LanPeers {
        let _lock = CONFIG.read().unwrap();
        match confy::load_path(Config::file_("_lan_peers")) {
            Ok(peers) => peers,
            Err(err) => {
                log::error!("Failed to load lan peers: {}", err);
                Default::default()
            }
        }
    }

    pub fn store(peers: &[DiscoveryPeer]) {
        let f = LanPeers {
            peers: peers.to_owned(),
        };
        if let Err(err) = store_path(Config::file_("_lan_peers"), f) {
            log::error!("Failed to store lan peers: {}", err);
        }
    }

    pub fn modify_time() -> crate::ResultType<u64> {
        let p = Config::file_("_lan_peers");
        Ok(fs::metadata(p)?
            .modified()?
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis() as _)
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct UserDefaultConfig {
    #[serde(default, deserialize_with = "deserialize_hashmap_string_string")]
    options: HashMap<String, String>,
}

impl UserDefaultConfig {
    fn read(key: &str) -> String {
        let mut cfg = USER_DEFAULT_CONFIG.write().unwrap();
        // we do so, because default config may changed in another process, but we don't sync it
        // but no need to read every time, give a small interval to avoid too many redundant read waste
        if cfg.1.elapsed() > Duration::from_secs(1) {
            *cfg = (Self::load(), Instant::now());
        }
        cfg.0.get(key)
    }

    pub fn load() -> UserDefaultConfig {
        Config::load_::<UserDefaultConfig>("_default")
    }

    #[inline]
    fn store(&self) {
        Config::store_(self, "_default");
    }

    pub fn get(&self, key: &str) -> String {
        match key {
            #[cfg(any(target_os = "android", target_os = "ios"))]
            keys::OPTION_VIEW_STYLE => self.get_string(key, "adaptive", vec!["original"]),
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            keys::OPTION_VIEW_STYLE => self.get_string(key, "original", vec!["adaptive"]),
            keys::OPTION_SCROLL_STYLE => self.get_string(key, "scrollauto", vec!["scrollbar"]),
            keys::OPTION_IMAGE_QUALITY => {
                self.get_string(key, "balanced", vec!["best", "low", "custom"])
            }
            keys::OPTION_CODEC_PREFERENCE => {
                self.get_string(key, "auto", vec!["vp8", "vp9", "av1", "h264", "h265"])
            }
            keys::OPTION_CUSTOM_IMAGE_QUALITY => self.get_num_string(key, 50.0, 10.0, 0xFFF as f64),
            keys::OPTION_CUSTOM_FPS => self.get_num_string(key, 30.0, 5.0, 120.0),
            keys::OPTION_ENABLE_FILE_COPY_PASTE => self.get_string(key, "Y", vec!["", "N"]),
            keys::OPTION_TRACKPAD_SPEED => self.get_num_string(key, 100, 10, 1000),
            _ => self
                .get_after(key)
                .map(|v| v.to_string())
                .unwrap_or_default(),
        }
    }

    pub fn set(&mut self, key: String, value: String) {
        if !is_option_can_save(
            &OVERWRITE_DISPLAY_SETTINGS,
            &key,
            &DEFAULT_DISPLAY_SETTINGS,
            &value,
        ) {
            return;
        }
        if value.is_empty() {
            self.options.remove(&key);
        } else {
            self.options.insert(key, value);
        }
        self.store();
    }

    #[inline]
    fn get_string(&self, key: &str, default: &str, others: Vec<&str>) -> String {
        match self.get_after(key) {
            Some(option) => {
                if others.contains(&option.as_str()) {
                    option.to_owned()
                } else {
                    default.to_owned()
                }
            }
            None => default.to_owned(),
        }
    }

    #[inline]
    fn get_num_string<T>(&self, key: &str, default: T, min: T, max: T) -> String
    where
        T: ToString + std::str::FromStr + std::cmp::PartialOrd + std::marker::Copy,
    {
        match self.get_after(key) {
            Some(option) => {
                let v: T = option.parse().unwrap_or(default);
                if v >= min && v <= max {
                    v.to_string()
                } else {
                    default.to_string()
                }
            }
            None => default.to_string(),
        }
    }

    fn get_after(&self, k: &str) -> Option<String> {
        get_or(
            &OVERWRITE_DISPLAY_SETTINGS,
            &self.options,
            &DEFAULT_DISPLAY_SETTINGS,
            k,
        )
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AbPeer {
    #[serde(
        default,
        deserialize_with = "deserialize_string",
        skip_serializing_if = "String::is_empty"
    )]
    pub id: String,
    #[serde(
        default,
        deserialize_with = "deserialize_string",
        skip_serializing_if = "String::is_empty"
    )]
    pub hash: String,
    #[serde(
        default,
        deserialize_with = "deserialize_string",
        skip_serializing_if = "String::is_empty"
    )]
    pub username: String,
    #[serde(
        default,
        deserialize_with = "deserialize_string",
        skip_serializing_if = "String::is_empty"
    )]
    pub hostname: String,
    #[serde(
        default,
        deserialize_with = "deserialize_string",
        skip_serializing_if = "String::is_empty"
    )]
    pub platform: String,
    #[serde(
        default,
        deserialize_with = "deserialize_string",
        skip_serializing_if = "String::is_empty"
    )]
    pub alias: String,
    #[serde(default, deserialize_with = "deserialize_vec_string")]
    pub tags: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AbEntry {
    #[serde(
        default,
        deserialize_with = "deserialize_string",
        skip_serializing_if = "String::is_empty"
    )]
    pub guid: String,
    #[serde(
        default,
        deserialize_with = "deserialize_string",
        skip_serializing_if = "String::is_empty"
    )]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_vec_abpeer")]
    pub peers: Vec<AbPeer>,
    #[serde(default, deserialize_with = "deserialize_vec_string")]
    pub tags: Vec<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_string",
        skip_serializing_if = "String::is_empty"
    )]
    pub tag_colors: String,
}

impl AbEntry {
    pub fn personal(&self) -> bool {
        self.name == "My address book" || self.name == "Legacy address book"
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Ab {
    #[serde(
        default,
        deserialize_with = "deserialize_string",
        skip_serializing_if = "String::is_empty"
    )]
    pub access_token: String,
    #[serde(default, deserialize_with = "deserialize_vec_abentry")]
    pub ab_entries: Vec<AbEntry>,
}

impl Ab {
    fn path() -> PathBuf {
        let filename = format!("{}_ab", APP_NAME.read().unwrap().clone());
        Config::path(filename)
    }

    pub fn store(json: String) {
        if let Ok(mut file) = std::fs::File::create(Self::path()) {
            let data = compress(json.as_bytes());
            let max_len = 64 * 1024 * 1024;
            if data.len() > max_len {
                // maxlen of function decompress
                log::error!("ab data too large, {} > {}", data.len(), max_len);
                return;
            }
            if let Ok(data) = symmetric_crypt(&data, true) {
                file.write_all(&data).ok();
            }
        };
    }

    pub fn load() -> Ab {
        if let Ok(mut file) = std::fs::File::open(Self::path()) {
            let mut data = vec![];
            if file.read_to_end(&mut data).is_ok() {
                if let Ok(data) = symmetric_crypt(&data, false) {
                    let data = decompress(&data);
                    if let Ok(ab) = serde_json::from_str::<Ab>(&String::from_utf8_lossy(&data)) {
                        return ab;
                    }
                }
            }
        };
        Self::remove();
        Ab::default()
    }

    pub fn remove() {
        std::fs::remove_file(Self::path()).ok();
    }
}

// use default value when field type is wrong
macro_rules! deserialize_default {
    ($func_name:ident, $return_type:ty) => {
        fn $func_name<'de, D>(deserializer: D) -> Result<$return_type, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            Ok(de::Deserialize::deserialize(deserializer).unwrap_or_default())
        }
    };
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct GroupPeer {
    #[serde(
        default,
        deserialize_with = "deserialize_string",
        skip_serializing_if = "String::is_empty"
    )]
    pub id: String,
    #[serde(
        default,
        deserialize_with = "deserialize_string",
        skip_serializing_if = "String::is_empty"
    )]
    pub username: String,
    #[serde(
        default,
        deserialize_with = "deserialize_string",
        skip_serializing_if = "String::is_empty"
    )]
    pub hostname: String,
    #[serde(
        default,
        deserialize_with = "deserialize_string",
        skip_serializing_if = "String::is_empty"
    )]
    pub platform: String,
    #[serde(
        default,
        deserialize_with = "deserialize_string",
        skip_serializing_if = "String::is_empty"
    )]
    pub login_name: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct GroupUser {
    #[serde(
        default,
        deserialize_with = "deserialize_string",
        skip_serializing_if = "String::is_empty"
    )]
    pub name: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct DeviceGroup {
    #[serde(
        default,
        deserialize_with = "deserialize_string",
        skip_serializing_if = "String::is_empty"
    )]
    pub name: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Group {
    #[serde(
        default,
        deserialize_with = "deserialize_string",
        skip_serializing_if = "String::is_empty"
    )]
    pub access_token: String,
    #[serde(default, deserialize_with = "deserialize_vec_groupuser")]
    pub users: Vec<GroupUser>,
    #[serde(default, deserialize_with = "deserialize_vec_grouppeer")]
    pub peers: Vec<GroupPeer>,
    #[serde(default, deserialize_with = "deserialize_vec_devicegroup")]
    pub device_groups: Vec<DeviceGroup>,
}

impl Group {
    fn path() -> PathBuf {
        let filename = format!("{}_group", APP_NAME.read().unwrap().clone());
        Config::path(filename)
    }

    pub fn store(json: String) {
        if let Ok(mut file) = std::fs::File::create(Self::path()) {
            let data = compress(json.as_bytes());
            let max_len = 64 * 1024 * 1024;
            if data.len() > max_len {
                // maxlen of function decompress
                return;
            }
            if let Ok(data) = symmetric_crypt(&data, true) {
                file.write_all(&data).ok();
            }
        };
    }

    pub fn load() -> Self {
        if let Ok(mut file) = std::fs::File::open(Self::path()) {
            let mut data = vec![];
            if file.read_to_end(&mut data).is_ok() {
                if let Ok(data) = symmetric_crypt(&data, false) {
                    let data = decompress(&data);
                    if let Ok(group) = serde_json::from_str::<Self>(&String::from_utf8_lossy(&data))
                    {
                        return group;
                    }
                }
            }
        };
        Self::remove();
        Self::default()
    }

    pub fn remove() {
        std::fs::remove_file(Self::path()).ok();
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct TrustedDevice {
    pub hwid: Bytes,
    pub time: i64,
    pub id: String,
    pub name: String,
    pub platform: String,
}

impl TrustedDevice {
    pub fn outdate(&self) -> bool {
        const DAYS_90: i64 = 90 * 24 * 60 * 60 * 1000;
        self.time + DAYS_90 < crate::get_time()
    }
}

deserialize_default!(deserialize_string, String);
deserialize_default!(deserialize_bool, bool);
deserialize_default!(deserialize_i32, i32);
deserialize_default!(deserialize_vec_u8, Vec<u8>);
deserialize_default!(deserialize_vec_string, Vec<String>);
deserialize_default!(deserialize_vec_i32_string_i32, Vec<(i32, String, i32)>);
deserialize_default!(deserialize_vec_discoverypeer, Vec<DiscoveryPeer>);
deserialize_default!(deserialize_vec_abpeer, Vec<AbPeer>);
deserialize_default!(deserialize_vec_abentry, Vec<AbEntry>);
deserialize_default!(deserialize_vec_groupuser, Vec<GroupUser>);
deserialize_default!(deserialize_vec_grouppeer, Vec<GroupPeer>);
deserialize_default!(deserialize_vec_devicegroup, Vec<DeviceGroup>);
deserialize_default!(deserialize_keypair, KeyPair);
deserialize_default!(deserialize_size, Size);
deserialize_default!(deserialize_hashmap_string_string, HashMap<String, String>);
deserialize_default!(deserialize_hashmap_string_bool,  HashMap<String, bool>);
deserialize_default!(deserialize_hashmap_resolutions, HashMap<String, Resolution>);

#[inline]
fn get_or(
    a: &RwLock<HashMap<String, String>>,
    b: &HashMap<String, String>,
    c: &RwLock<HashMap<String, String>>,
    k: &str,
) -> Option<String> {
    a.read()
        .unwrap()
        .get(k)
        .or(b.get(k))
        .or(c.read().unwrap().get(k))
        .cloned()
}

#[inline]
fn is_option_can_save(
    overwrite: &RwLock<HashMap<String, String>>,
    k: &str,
    defaults: &RwLock<HashMap<String, String>>,
    v: &str,
) -> bool {
    if overwrite.read().unwrap().contains_key(k)
        || defaults.read().unwrap().get(k).map_or(false, |x| x == v)
    {
        return false;
    }
    true
}

#[inline]
pub fn is_incoming_only() -> bool {
    HARD_SETTINGS
        .read()
        .unwrap()
        .get("conn-type")
        .map_or(false, |x| x == ("incoming"))
}

#[inline]
pub fn is_outgoing_only() -> bool {
    HARD_SETTINGS
        .read()
        .unwrap()
        .get("conn-type")
        .map_or(false, |x| x == ("outgoing"))
}

#[inline]
fn is_some_hard_opton(name: &str) -> bool {
    HARD_SETTINGS
        .read()
        .unwrap()
        .get(name)
        .map_or(false, |x| x == ("Y"))
}

#[inline]
pub fn is_disable_tcp_listen() -> bool {
    is_some_hard_opton("disable-tcp-listen")
}

#[inline]
pub fn is_disable_settings() -> bool {
    is_some_hard_opton("disable-settings")
}

#[inline]
pub fn is_disable_ab() -> bool {
    is_some_hard_opton("disable-ab")
}

#[inline]
pub fn is_disable_account() -> bool {
    is_some_hard_opton("disable-account")
}

#[inline]
pub fn is_disable_installation() -> bool {
    is_some_hard_opton("disable-installation")
}

// This function must be kept the same as the one in flutter and sciter code.
// flutter: flutter/lib/common.dart -> option2bool()
// sciter: Does not have the function, but it should be kept the same.
pub fn option2bool(option: &str, value: &str) -> bool {
    if option.starts_with("enable-") {
        value != "N"
    } else if option.starts_with("allow-")
        || option == "stop-service"
        || option == keys::OPTION_DIRECT_SERVER
        || option == "force-always-relay"
    {
        value == "Y"
    } else {
        value != "N"
    }
}

pub fn use_ws() -> bool {
    let option = keys::OPTION_ALLOW_WEBSOCKET;
    option2bool(option, &Config::get_option(option))
}

pub mod keys {
    pub const OPTION_VIEW_ONLY: &str = "view_only";
    pub const OPTION_SHOW_MONITORS_TOOLBAR: &str = "show_monitors_toolbar";
    pub const OPTION_COLLAPSE_TOOLBAR: &str = "collapse_toolbar";
    pub const OPTION_SHOW_REMOTE_CURSOR: &str = "show_remote_cursor";
    pub const OPTION_FOLLOW_REMOTE_CURSOR: &str = "follow_remote_cursor";
    pub const OPTION_FOLLOW_REMOTE_WINDOW: &str = "follow_remote_window";
    pub const OPTION_ZOOM_CURSOR: &str = "zoom-cursor";
    pub const OPTION_SHOW_QUALITY_MONITOR: &str = "show_quality_monitor";
    pub const OPTION_DISABLE_AUDIO: &str = "disable_audio";
    pub const OPTION_ENABLE_REMOTE_PRINTER: &str = "enable-remote-printer";
    pub const OPTION_ENABLE_FILE_COPY_PASTE: &str = "enable-file-copy-paste";
    pub const OPTION_DISABLE_CLIPBOARD: &str = "disable_clipboard";
    pub const OPTION_LOCK_AFTER_SESSION_END: &str = "lock_after_session_end";
    pub const OPTION_PRIVACY_MODE: &str = "privacy_mode";
    pub const OPTION_TOUCH_MODE: &str = "touch-mode";
    pub const OPTION_I444: &str = "i444";
    pub const OPTION_REVERSE_MOUSE_WHEEL: &str = "reverse_mouse_wheel";
    pub const OPTION_SWAP_LEFT_RIGHT_MOUSE: &str = "swap-left-right-mouse";
    pub const OPTION_DISPLAYS_AS_INDIVIDUAL_WINDOWS: &str = "displays_as_individual_windows";
    pub const OPTION_USE_ALL_MY_DISPLAYS_FOR_THE_REMOTE_SESSION: &str =
        "use_all_my_displays_for_the_remote_session";
    pub const OPTION_VIEW_STYLE: &str = "view_style";
    pub const OPTION_SCROLL_STYLE: &str = "scroll_style";
    pub const OPTION_IMAGE_QUALITY: &str = "image_quality";
    pub const OPTION_CUSTOM_IMAGE_QUALITY: &str = "custom_image_quality";
    pub const OPTION_CUSTOM_FPS: &str = "custom-fps";
    pub const OPTION_CODEC_PREFERENCE: &str = "codec-preference";
    pub const OPTION_SYNC_INIT_CLIPBOARD: &str = "sync-init-clipboard";
    pub const OPTION_THEME: &str = "theme";
    pub const OPTION_LANGUAGE: &str = "lang";
    pub const OPTION_REMOTE_MENUBAR_DRAG_LEFT: &str = "remote-menubar-drag-left";
    pub const OPTION_REMOTE_MENUBAR_DRAG_RIGHT: &str = "remote-menubar-drag-right";
    pub const OPTION_HIDE_AB_TAGS_PANEL: &str = "hideAbTagsPanel";
    pub const OPTION_ENABLE_CONFIRM_CLOSING_TABS: &str = "enable-confirm-closing-tabs";
    pub const OPTION_ENABLE_OPEN_NEW_CONNECTIONS_IN_TABS: &str =
        "enable-open-new-connections-in-tabs";
    pub const OPTION_TEXTURE_RENDER: &str = "use-texture-render";
    pub const OPTION_ALLOW_D3D_RENDER: &str = "allow-d3d-render";
    pub const OPTION_ENABLE_CHECK_UPDATE: &str = "enable-check-update";
    pub const OPTION_ALLOW_AUTO_UPDATE: &str = "allow-auto-update";
    pub const OPTION_SYNC_AB_WITH_RECENT_SESSIONS: &str = "sync-ab-with-recent-sessions";
    pub const OPTION_SYNC_AB_TAGS: &str = "sync-ab-tags";
    pub const OPTION_FILTER_AB_BY_INTERSECTION: &str = "filter-ab-by-intersection";
    pub const OPTION_ACCESS_MODE: &str = "access-mode";
    pub const OPTION_ENABLE_KEYBOARD: &str = "enable-keyboard";
    pub const OPTION_ENABLE_CLIPBOARD: &str = "enable-clipboard";
    pub const OPTION_ENABLE_FILE_TRANSFER: &str = "enable-file-transfer";
    pub const OPTION_ENABLE_CAMERA: &str = "enable-camera";
    pub const OPTION_ENABLE_TERMINAL: &str = "enable-terminal";
    pub const OPTION_TERMINAL_PERSISTENT: &str = "terminal-persistent";
    pub const OPTION_ENABLE_AUDIO: &str = "enable-audio";
    pub const OPTION_ENABLE_TUNNEL: &str = "enable-tunnel";
    pub const OPTION_ENABLE_REMOTE_RESTART: &str = "enable-remote-restart";
    pub const OPTION_ENABLE_RECORD_SESSION: &str = "enable-record-session";
    pub const OPTION_ENABLE_BLOCK_INPUT: &str = "enable-block-input";
    pub const OPTION_ALLOW_REMOTE_CONFIG_MODIFICATION: &str = "allow-remote-config-modification";
    pub const OPTION_ALLOW_NUMERNIC_ONE_TIME_PASSWORD: &str = "allow-numeric-one-time-password";
    pub const OPTION_ENABLE_LAN_DISCOVERY: &str = "enable-lan-discovery";
    pub const OPTION_DIRECT_SERVER: &str = "direct-server";
    pub const OPTION_DIRECT_ACCESS_PORT: &str = "direct-access-port";
    pub const OPTION_WHITELIST: &str = "whitelist";
    pub const OPTION_ALLOW_AUTO_DISCONNECT: &str = "allow-auto-disconnect";
    pub const OPTION_AUTO_DISCONNECT_TIMEOUT: &str = "auto-disconnect-timeout";
    pub const OPTION_ALLOW_ONLY_CONN_WINDOW_OPEN: &str = "allow-only-conn-window-open";
    pub const OPTION_ALLOW_AUTO_RECORD_INCOMING: &str = "allow-auto-record-incoming";
    pub const OPTION_ALLOW_AUTO_RECORD_OUTGOING: &str = "allow-auto-record-outgoing";
    pub const OPTION_VIDEO_SAVE_DIRECTORY: &str = "video-save-directory";
    pub const OPTION_ENABLE_ABR: &str = "enable-abr";
    pub const OPTION_ALLOW_REMOVE_WALLPAPER: &str = "allow-remove-wallpaper";
    pub const OPTION_ALLOW_ALWAYS_SOFTWARE_RENDER: &str = "allow-always-software-render";
    pub const OPTION_ALLOW_LINUX_HEADLESS: &str = "allow-linux-headless";
    pub const OPTION_ENABLE_HWCODEC: &str = "enable-hwcodec";
    pub const OPTION_APPROVE_MODE: &str = "approve-mode";
    pub const OPTION_VERIFICATION_METHOD: &str = "verification-method";
    pub const OPTION_TEMPORARY_PASSWORD_LENGTH: &str = "temporary-password-length";
    pub const OPTION_CUSTOM_RENDEZVOUS_SERVER: &str = "custom-rendezvous-server";
    pub const OPTION_API_SERVER: &str = "api-server";
    pub const OPTION_KEY: &str = "key";
    pub const OPTION_ALLOW_WEBSOCKET: &str = "allow-websocket";
    pub const OPTION_PRESET_ADDRESS_BOOK_NAME: &str = "preset-address-book-name";
    pub const OPTION_PRESET_ADDRESS_BOOK_TAG: &str = "preset-address-book-tag";
    pub const OPTION_ENABLE_DIRECTX_CAPTURE: &str = "enable-directx-capture";
    pub const OPTION_ENABLE_ANDROID_SOFTWARE_ENCODING_HALF_SCALE: &str =
        "enable-android-software-encoding-half-scale";
    pub const OPTION_ENABLE_TRUSTED_DEVICES: &str = "enable-trusted-devices";
    pub const OPTION_AV1_TEST: &str = "av1-test";
    pub const OPTION_TRACKPAD_SPEED: &str = "trackpad-speed";
    pub const OPTION_REGISTER_DEVICE: &str = "register-device";

    // built-in options
    pub const OPTION_DISPLAY_NAME: &str = "display-name";
    pub const OPTION_DISABLE_UDP: &str = "disable-udp";
    pub const OPTION_PRESET_DEVICE_GROUP_NAME: &str = "preset-device-group-name";
    pub const OPTION_PRESET_USERNAME: &str = "preset-user-name";
    pub const OPTION_PRESET_STRATEGY_NAME: &str = "preset-strategy-name";
    pub const OPTION_REMOVE_PRESET_PASSWORD_WARNING: &str = "remove-preset-password-warning";
    pub const OPTION_HIDE_SECURITY_SETTINGS: &str = "hide-security-settings";
    pub const OPTION_HIDE_NETWORK_SETTINGS: &str = "hide-network-settings";
    pub const OPTION_HIDE_SERVER_SETTINGS: &str = "hide-server-settings";
    pub const OPTION_HIDE_PROXY_SETTINGS: &str = "hide-proxy-settings";
    pub const OPTION_HIDE_REMOTE_PRINTER_SETTINGS: &str = "hide-remote-printer-settings";
    pub const OPTION_HIDE_WEBSOCKET_SETTINGS: &str = "hide-websocket-settings";

    // Connection punch-through options
    pub const OPTION_ENABLE_UDP_PUNCH: &str = "enable-udp-punch";
    pub const OPTION_ENABLE_IPV6_PUNCH: &str = "enable-ipv6-punch";
    pub const OPTION_HIDE_USERNAME_ON_CARD: &str = "hide-username-on-card";
    pub const OPTION_HIDE_HELP_CARDS: &str = "hide-help-cards";
    pub const OPTION_DEFAULT_CONNECT_PASSWORD: &str = "default-connect-password";
    pub const OPTION_HIDE_TRAY: &str = "hide-tray";
    pub const OPTION_ONE_WAY_CLIPBOARD_REDIRECTION: &str = "one-way-clipboard-redirection";
    pub const OPTION_ALLOW_LOGON_SCREEN_PASSWORD: &str = "allow-logon-screen-password";
    pub const OPTION_ONE_WAY_FILE_TRANSFER: &str = "one-way-file-transfer";
    pub const OPTION_ALLOW_HTTPS_21114: &str = "allow-https-2114";
    pub const OPTION_ALLOW_HOSTNAME_AS_ID: &str = "allow-hostname-as-id";
    pub const OPTION_HIDE_POWERED_BY_ME: &str = "hide-powered-by-me";
    pub const OPTION_MAIN_WINDOW_ALWAYS_ON_TOP: &str = "main-window-always-on-top";

    // flutter local options
    pub const OPTION_FLUTTER_REMOTE_MENUBAR_STATE: &str = "remoteMenubarState";
    pub const OPTION_FLUTTER_PEER_SORTING: &str = "peer-sorting";
    pub const OPTION_FLUTTER_PEER_TAB_INDEX: &str = "peer-tab-index";
    pub const OPTION_FLUTTER_PEER_TAB_ORDER: &str = "peer-tab-order";
    pub const OPTION_FLUTTER_PEER_TAB_VISIBLE: &str = "peer-tab-visible";
    pub const OPTION_FLUTTER_PEER_CARD_UI_TYLE: &str = "peer-card-ui-type";
    pub const OPTION_FLUTTER_CURRENT_AB_NAME: &str = "current-ab-name";
    pub const OPTION_ALLOW_REMOTE_CM_MODIFICATION: &str = "allow-remote-cm-modification";

    pub const OPTION_PRINTER_INCOMING_JOB_ACTION: &str = "printer-incomming-job-action";
    pub const OPTION_PRINTER_ALLOW_AUTO_PRINT: &str = "allow-printer-auto-print";
    pub const OPTION_PRINTER_SELECTED_NAME: &str = "printer-selected-name";

    // android floating window options
    pub const OPTION_DISABLE_FLOATING_WINDOW: &str = "disable-floating-window";
    pub const OPTION_FLOATING_WINDOW_SIZE: &str = "floating-window-size";
    pub const OPTION_FLOATING_WINDOW_UNTOUCHABLE: &str = "floating-window-untouchable";
    pub const OPTION_FLOATING_WINDOW_TRANSPARENCY: &str = "floating-window-transparency";
    pub const OPTION_FLOATING_WINDOW_SVG: &str = "floating-window-svg";

    // android keep screen on
    pub const OPTION_KEEP_SCREEN_ON: &str = "keep-screen-on";

    pub const OPTION_DISABLE_GROUP_PANEL: &str = "disable-group-panel";
    pub const OPTION_DISABLE_DISCOVERY_PANEL: &str = "disable-discovery-panel";
    pub const OPTION_PRE_ELEVATE_SERVICE: &str = "pre-elevate-service";

    // proxy settings
    // The following options are not real keys, they are just used for custom client advanced settings.
    // The real keys are in Config2::socks.
    pub const OPTION_PROXY_URL: &str = "proxy-url";
    pub const OPTION_PROXY_USERNAME: &str = "proxy-username";
    pub const OPTION_PROXY_PASSWORD: &str = "proxy-password";

    // DEFAULT_DISPLAY_SETTINGS, OVERWRITE_DISPLAY_SETTINGS
    pub const KEYS_DISPLAY_SETTINGS: &[&str] = &[
        OPTION_VIEW_ONLY,
        OPTION_SHOW_MONITORS_TOOLBAR,
        OPTION_COLLAPSE_TOOLBAR,
        OPTION_SHOW_REMOTE_CURSOR,
        OPTION_FOLLOW_REMOTE_CURSOR,
        OPTION_FOLLOW_REMOTE_WINDOW,
        OPTION_ZOOM_CURSOR,
        OPTION_SHOW_QUALITY_MONITOR,
        OPTION_DISABLE_AUDIO,
        OPTION_ENABLE_FILE_COPY_PASTE,
        OPTION_DISABLE_CLIPBOARD,
        OPTION_LOCK_AFTER_SESSION_END,
        OPTION_PRIVACY_MODE,
        OPTION_TOUCH_MODE,
        OPTION_I444,
        OPTION_REVERSE_MOUSE_WHEEL,
        OPTION_SWAP_LEFT_RIGHT_MOUSE,
        OPTION_DISPLAYS_AS_INDIVIDUAL_WINDOWS,
        OPTION_USE_ALL_MY_DISPLAYS_FOR_THE_REMOTE_SESSION,
        OPTION_VIEW_STYLE,
        OPTION_TERMINAL_PERSISTENT,
        OPTION_SCROLL_STYLE,
        OPTION_IMAGE_QUALITY,
        OPTION_CUSTOM_IMAGE_QUALITY,
        OPTION_CUSTOM_FPS,
        OPTION_CODEC_PREFERENCE,
        OPTION_SYNC_INIT_CLIPBOARD,
        OPTION_TRACKPAD_SPEED,
    ];
    // DEFAULT_LOCAL_SETTINGS, OVERWRITE_LOCAL_SETTINGS
    pub const KEYS_LOCAL_SETTINGS: &[&str] = &[
        OPTION_THEME,
        OPTION_LANGUAGE,
        OPTION_ENABLE_CONFIRM_CLOSING_TABS,
        OPTION_ENABLE_OPEN_NEW_CONNECTIONS_IN_TABS,
        OPTION_TEXTURE_RENDER,
        OPTION_ALLOW_D3D_RENDER,
        OPTION_SYNC_AB_WITH_RECENT_SESSIONS,
        OPTION_SYNC_AB_TAGS,
        OPTION_FILTER_AB_BY_INTERSECTION,
        OPTION_REMOTE_MENUBAR_DRAG_LEFT,
        OPTION_REMOTE_MENUBAR_DRAG_RIGHT,
        OPTION_HIDE_AB_TAGS_PANEL,
        OPTION_FLUTTER_REMOTE_MENUBAR_STATE,
        OPTION_FLUTTER_PEER_SORTING,
        OPTION_FLUTTER_PEER_TAB_INDEX,
        OPTION_FLUTTER_PEER_TAB_ORDER,
        OPTION_FLUTTER_PEER_TAB_VISIBLE,
        OPTION_FLUTTER_PEER_CARD_UI_TYLE,
        OPTION_FLUTTER_CURRENT_AB_NAME,
        OPTION_DISABLE_FLOATING_WINDOW,
        OPTION_FLOATING_WINDOW_SIZE,
        OPTION_FLOATING_WINDOW_UNTOUCHABLE,
        OPTION_FLOATING_WINDOW_TRANSPARENCY,
        OPTION_FLOATING_WINDOW_SVG,
        OPTION_KEEP_SCREEN_ON,
        OPTION_DISABLE_GROUP_PANEL,
        OPTION_DISABLE_DISCOVERY_PANEL,
        OPTION_PRE_ELEVATE_SERVICE,
        OPTION_ALLOW_REMOTE_CM_MODIFICATION,
        OPTION_ALLOW_AUTO_RECORD_OUTGOING,
        OPTION_VIDEO_SAVE_DIRECTORY,
        OPTION_ENABLE_UDP_PUNCH,
        OPTION_ENABLE_IPV6_PUNCH,
    ];
    // DEFAULT_SETTINGS, OVERWRITE_SETTINGS
    pub const KEYS_SETTINGS: &[&str] = &[
        OPTION_ACCESS_MODE,
        OPTION_ENABLE_KEYBOARD,
        OPTION_ENABLE_CLIPBOARD,
        OPTION_ENABLE_FILE_TRANSFER,
        OPTION_ENABLE_CAMERA,
        OPTION_ENABLE_TERMINAL,
        OPTION_ENABLE_REMOTE_PRINTER,
        OPTION_ENABLE_AUDIO,
        OPTION_ENABLE_TUNNEL,
        OPTION_ENABLE_REMOTE_RESTART,
        OPTION_ENABLE_RECORD_SESSION,
        OPTION_ENABLE_BLOCK_INPUT,
        OPTION_ALLOW_REMOTE_CONFIG_MODIFICATION,
        OPTION_ALLOW_NUMERNIC_ONE_TIME_PASSWORD,
        OPTION_ENABLE_LAN_DISCOVERY,
        OPTION_DIRECT_SERVER,
        OPTION_DIRECT_ACCESS_PORT,
        OPTION_WHITELIST,
        OPTION_ALLOW_AUTO_DISCONNECT,
        OPTION_AUTO_DISCONNECT_TIMEOUT,
        OPTION_ALLOW_ONLY_CONN_WINDOW_OPEN,
        OPTION_ALLOW_AUTO_RECORD_INCOMING,
        OPTION_ENABLE_ABR,
        OPTION_ALLOW_REMOVE_WALLPAPER,
        OPTION_ALLOW_ALWAYS_SOFTWARE_RENDER,
        OPTION_ALLOW_LINUX_HEADLESS,
        OPTION_ENABLE_HWCODEC,
        OPTION_APPROVE_MODE,
        OPTION_VERIFICATION_METHOD,
        OPTION_TEMPORARY_PASSWORD_LENGTH,
        OPTION_PROXY_URL,
        OPTION_PROXY_USERNAME,
        OPTION_PROXY_PASSWORD,
        OPTION_CUSTOM_RENDEZVOUS_SERVER,
        OPTION_API_SERVER,
        OPTION_KEY,
        OPTION_ALLOW_WEBSOCKET,
        OPTION_PRESET_ADDRESS_BOOK_NAME,
        OPTION_PRESET_ADDRESS_BOOK_TAG,
        OPTION_ENABLE_DIRECTX_CAPTURE,
        OPTION_ENABLE_ANDROID_SOFTWARE_ENCODING_HALF_SCALE,
        OPTION_ENABLE_TRUSTED_DEVICES,
    ];

    // BUILDIN_SETTINGS
    pub const KEYS_BUILDIN_SETTINGS: &[&str] = &[
        OPTION_DISPLAY_NAME,
        OPTION_DISABLE_UDP,
        OPTION_PRESET_DEVICE_GROUP_NAME,
        OPTION_PRESET_USERNAME,
        OPTION_PRESET_STRATEGY_NAME,
        OPTION_REMOVE_PRESET_PASSWORD_WARNING,
        OPTION_HIDE_SECURITY_SETTINGS,
        OPTION_HIDE_NETWORK_SETTINGS,
        OPTION_HIDE_SERVER_SETTINGS,
        OPTION_HIDE_PROXY_SETTINGS,
        OPTION_HIDE_REMOTE_PRINTER_SETTINGS,
        OPTION_HIDE_WEBSOCKET_SETTINGS,
        OPTION_HIDE_USERNAME_ON_CARD,
        OPTION_HIDE_HELP_CARDS,
        OPTION_DEFAULT_CONNECT_PASSWORD,
        OPTION_HIDE_TRAY,
        OPTION_ONE_WAY_CLIPBOARD_REDIRECTION,
        OPTION_ALLOW_LOGON_SCREEN_PASSWORD,
        OPTION_ONE_WAY_FILE_TRANSFER,
        OPTION_ALLOW_HTTPS_21114,
        OPTION_ALLOW_HOSTNAME_AS_ID,
        OPTION_REGISTER_DEVICE,
        OPTION_HIDE_POWERED_BY_ME,
        OPTION_MAIN_WINDOW_ALWAYS_ON_TOP,
    ];
}

pub fn common_load<
    T: serde::Serialize + serde::de::DeserializeOwned + Default + std::fmt::Debug,
>(
    suffix: &str,
) -> T {
    Config::load_::<T>(suffix)
}

pub fn common_store<T: serde::Serialize>(config: &T, suffix: &str) {
    Config::store_(config, suffix);
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Status {
    #[serde(default, deserialize_with = "deserialize_hashmap_string_string")]
    values: HashMap<String, String>,
}

impl Status {
    fn load() -> Status {
        Config::load_::<Status>("_status")
    }

    fn store(&self) {
        Config::store_(self, "_status");
    }

    pub fn get(k: &str) -> String {
        STATUS
            .read()
            .unwrap()
            .values
            .get(k)
            .cloned()
            .unwrap_or_default()
    }

    pub fn set(k: &str, v: String) {
        if Self::get(k) == v {
            return;
        }

        let mut st = STATUS.write().unwrap();
        st.values.insert(k.to_owned(), v);
        st.store();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        let cfg: Config = Default::default();
        let res = toml::to_string_pretty(&cfg);
        assert!(res.is_ok());
        let cfg: PeerConfig = Default::default();
        let res = toml::to_string_pretty(&cfg);
        assert!(res.is_ok());
    }

    #[test]
    fn test_overwrite_settings() {
        DEFAULT_SETTINGS
            .write()
            .unwrap()
            .insert("b".to_string(), "a".to_string());
        DEFAULT_SETTINGS
            .write()
            .unwrap()
            .insert("c".to_string(), "a".to_string());
        CONFIG2
            .write()
            .unwrap()
            .options
            .insert("a".to_string(), "b".to_string());
        CONFIG2
            .write()
            .unwrap()
            .options
            .insert("b".to_string(), "b".to_string());
        OVERWRITE_SETTINGS
            .write()
            .unwrap()
            .insert("b".to_string(), "c".to_string());
        OVERWRITE_SETTINGS
            .write()
            .unwrap()
            .insert("c".to_string(), "f".to_string());
        OVERWRITE_SETTINGS
            .write()
            .unwrap()
            .insert("d".to_string(), "c".to_string());
        let mut res: HashMap<String, String> = Default::default();
        res.insert("b".to_owned(), "c".to_string());
        res.insert("d".to_owned(), "c".to_string());
        res.insert("c".to_owned(), "a".to_string());
        Config::purify_options(&mut res);
        assert!(res.len() == 0);
        res.insert("b".to_owned(), "c".to_string());
        res.insert("d".to_owned(), "c".to_string());
        res.insert("c".to_owned(), "a".to_string());
        res.insert("f".to_owned(), "a".to_string());
        Config::purify_options(&mut res);
        assert!(res.len() == 1);
        res.insert("b".to_owned(), "c".to_string());
        res.insert("d".to_owned(), "c".to_string());
        res.insert("c".to_owned(), "a".to_string());
        res.insert("f".to_owned(), "a".to_string());
        res.insert("e".to_owned(), "d".to_string());
        Config::purify_options(&mut res);
        assert!(res.len() == 2);
        res.insert("b".to_owned(), "c".to_string());
        res.insert("d".to_owned(), "c".to_string());
        res.insert("c".to_owned(), "a".to_string());
        res.insert("f".to_owned(), "a".to_string());
        res.insert("c".to_owned(), "d".to_string());
        res.insert("d".to_owned(), "cc".to_string());
        Config::purify_options(&mut res);
        DEFAULT_SETTINGS
            .write()
            .unwrap()
            .insert("f".to_string(), "c".to_string());
        Config::purify_options(&mut res);
        assert!(res.len() == 2);
        DEFAULT_SETTINGS
            .write()
            .unwrap()
            .insert("f".to_string(), "a".to_string());
        Config::purify_options(&mut res);
        assert!(res.len() == 1);
        let res = Config::get_options();
        assert!(res["a"] == "b");
        assert!(res["c"] == "f");
        assert!(res["b"] == "c");
        assert!(res["d"] == "c");
        assert!(Config::get_option("a") == "b");
        assert!(Config::get_option("c") == "f");
        assert!(Config::get_option("b") == "c");
        assert!(Config::get_option("d") == "c");
        DEFAULT_SETTINGS.write().unwrap().clear();
        OVERWRITE_SETTINGS.write().unwrap().clear();
        CONFIG2.write().unwrap().options.clear();

        DEFAULT_LOCAL_SETTINGS
            .write()
            .unwrap()
            .insert("b".to_string(), "a".to_string());
        DEFAULT_LOCAL_SETTINGS
            .write()
            .unwrap()
            .insert("c".to_string(), "a".to_string());
        LOCAL_CONFIG
            .write()
            .unwrap()
            .options
            .insert("a".to_string(), "b".to_string());
        LOCAL_CONFIG
            .write()
            .unwrap()
            .options
            .insert("b".to_string(), "b".to_string());
        OVERWRITE_LOCAL_SETTINGS
            .write()
            .unwrap()
            .insert("b".to_string(), "c".to_string());
        OVERWRITE_LOCAL_SETTINGS
            .write()
            .unwrap()
            .insert("d".to_string(), "c".to_string());
        assert!(LocalConfig::get_option("a") == "b");
        assert!(LocalConfig::get_option("c") == "a");
        assert!(LocalConfig::get_option("b") == "c");
        assert!(LocalConfig::get_option("d") == "c");
        DEFAULT_LOCAL_SETTINGS.write().unwrap().clear();
        OVERWRITE_LOCAL_SETTINGS.write().unwrap().clear();
        LOCAL_CONFIG.write().unwrap().options.clear();

        DEFAULT_DISPLAY_SETTINGS
            .write()
            .unwrap()
            .insert("b".to_string(), "a".to_string());
        DEFAULT_DISPLAY_SETTINGS
            .write()
            .unwrap()
            .insert("c".to_string(), "a".to_string());
        USER_DEFAULT_CONFIG
            .write()
            .unwrap()
            .0
            .options
            .insert("a".to_string(), "b".to_string());
        USER_DEFAULT_CONFIG
            .write()
            .unwrap()
            .0
            .options
            .insert("b".to_string(), "b".to_string());
        OVERWRITE_DISPLAY_SETTINGS
            .write()
            .unwrap()
            .insert("b".to_string(), "c".to_string());
        OVERWRITE_DISPLAY_SETTINGS
            .write()
            .unwrap()
            .insert("d".to_string(), "c".to_string());
        assert!(UserDefaultConfig::read("a") == "b");
        assert!(UserDefaultConfig::read("c") == "a");
        assert!(UserDefaultConfig::read("b") == "c");
        assert!(UserDefaultConfig::read("d") == "c");
        DEFAULT_DISPLAY_SETTINGS.write().unwrap().clear();
        OVERWRITE_DISPLAY_SETTINGS.write().unwrap().clear();
        LOCAL_CONFIG.write().unwrap().options.clear();
    }

    #[test]
    fn test_config_deserialize() {
        let wrong_type_str = r#"
        id = true
        enc_id = []
        password = 1
        salt = "123456"
        key_pair = {}
        key_confirmed = "1"
        keys_confirmed = 1
        "#;
        let cfg = toml::from_str::<Config>(wrong_type_str);
        assert_eq!(
            cfg,
            Ok(Config {
                salt: "123456".to_string(),
                ..Default::default()
            })
        );

        let wrong_field_str = r#"
        hello = "world"
        key_confirmed = true
        "#;
        let cfg = toml::from_str::<Config>(wrong_field_str);
        assert_eq!(
            cfg,
            Ok(Config {
                key_confirmed: true,
                ..Default::default()
            })
        );
    }

    #[test]
    fn test_peer_config_deserialize() {
        let default_peer_config = toml::from_str::<PeerConfig>("").unwrap();
        // test custom_resolution
        {
            let wrong_type_str = r#"
            view_style = "adaptive"
            scroll_style = "scrollbar"
            custom_resolutions = true
            "#;
            let mut cfg_to_compare = default_peer_config.clone();
            cfg_to_compare.view_style = "adaptive".to_string();
            cfg_to_compare.scroll_style = "scrollbar".to_string();
            let cfg = toml::from_str::<PeerConfig>(wrong_type_str);
            assert_eq!(cfg, Ok(cfg_to_compare), "Failed to test wrong_type_str");

            let wrong_type_str = r#"
            view_style = "adaptive"
            scroll_style = "scrollbar"
            [custom_resolutions.0]
            w = "1920"
            h = 1080
            "#;
            let mut cfg_to_compare = default_peer_config.clone();
            cfg_to_compare.view_style = "adaptive".to_string();
            cfg_to_compare.scroll_style = "scrollbar".to_string();
            let cfg = toml::from_str::<PeerConfig>(wrong_type_str);
            assert_eq!(cfg, Ok(cfg_to_compare), "Failed to test wrong_type_str");

            let wrong_field_str = r#"
            [custom_resolutions.0]
            w = 1920
            h = 1080
            hello = "world"
            [ui_flutter]
            "#;
            let mut cfg_to_compare = default_peer_config.clone();
            cfg_to_compare.custom_resolutions =
                HashMap::from([("0".to_string(), Resolution { w: 1920, h: 1080 })]);
            let cfg = toml::from_str::<PeerConfig>(wrong_field_str);
            assert_eq!(cfg, Ok(cfg_to_compare), "Failed to test wrong_field_str");
        }
    }

    #[test]
    fn test_store_load() {
        let peerconfig_id = "123456789";
        let cfg: PeerConfig = Default::default();
        cfg.store(&peerconfig_id);
        assert_eq!(PeerConfig::load(&peerconfig_id), cfg);

        #[cfg(not(windows))]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                // ignore file type information by masking with 0o777 (see https://stackoverflow.com/a/50045872)
                fs::metadata(PeerConfig::path(&peerconfig_id))
                    .expect("reading metadata failed")
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }
    }
}
