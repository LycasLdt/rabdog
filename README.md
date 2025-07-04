<div align="center">

## :dog: Rabdog 
</div>

简简单单，获取个 **Scratch社区作品**。

> [!WARNING]
> 仍处于**超前版本**，请谨慎使用
>
> 凡使用此程序**违反社区规定**的，此程序**不承担任何责任**

### :rocket:支持
||:label: 支持状态|:rotating_light: 注意|
|-|-|-|
|**[Scratch]**|:warning:|使用 [**TurboWarp Trampoline**](https://trampoline.turbowarp.org) 和 [**Chilipar**](https://chilipar.alibga.icu) (**自建**，部署于 `Vercel`) 代理，**稳定性不能保证**|
|**[CCW]**|:white_check_mark:||
|**[Cocrea World][cocrea-world]**|:white_check_mark:||
|**[Clipcc]**|:warning:|Rabdog 中 **使用的库 `rsa v0.10.0-pre.1` 会受 [The Marvin Attack](https://people.redhat.com/~hkario/marvin/) 影响** |
|**[小码王][xmw]**|:white_check_mark:||
|**[Scratch 中社][scratch-cn]**|:white_check_mark:|从 `v0.2.1` 开始支持|
|**[稽木世界 / 阿尔法营][gitblock]**|:construction:|从 `v0.3.0` 开始支持，**限流作品无法下载**|
|**[40code]**|:no_entry:|**不可使用，仍未修复**|

<dd>
:white_check_mark: 全部支持

:construction: 支持但仍存在问题

:warning: :rotating_light: 有安全问题 :rotating_light:，**不建议使用**

:no_entry: 不再支持
</dd>

### :package:安装

[![ci](https://github.com/LycasLdt/rabdog/actions/workflows/ci.yml/badge.svg)](https://github.com/LycasLdt/rabdog/actions/workflows/ci.yml)

- 在 [Releases][download] 中下载 **发布版本** 
- 在 [Actions][actions] 中 CI 上传的 `Artifacts` 下载 **测试版本**

#### 手动下载

运行:

```
$ git clone https://github.com/LycasLdt/rabdog

$ cargo build --bin rabdog
```

### :white_check_mark:使用

#### 单链接下载

```bash
$ rabdog "https://www.ccw.site/detail/65b9182433db685782f24f8f"

  [共创世界 [65b9182433db685782f24f8f]] 下载完成
```

#### 多链接下载

```bash
$ rabdog "https://www.ccw.site/detail/65b9182433db685782f24f8f"
> "https://codingclip.com/project/114"
> "https://world.xiaomawang.com/community/main/compose/KmCD666J"

  [共创世界 [65b9182433db685782f24f8f]] 下载完成
  [Clipcc [114]] 下载完成
  [小码王 [KmCD666J]] 下载完成
```

#### 指定下载位置

```bash
$ rabdog --path ~ "https://www.ccw.site/detail/65b9182433db685782f24f8f"
```

#### 不在终端输出

```bash
$ rabdog --silent

$ 
```

### :heart_on_fire:贡献

欢迎`PR`！

### :key:许可证
MIT

[download]: https://github.com/LycasLdt/rabdog/releases
[actions]: https://github.com/LycasLdt/rabdog/actions

[scratch]: https://scratch.mit.edu/
[ccw]: https://www.ccw.site
[cocrea-world]: https://www.cocrea.world
[clipcc]: https://codingclip.com
[40code]: https://40code.com
[xmw]: https://world.xiaomawang.com/
[scratch-cn]: https://www.scratch-cn.cn/
[40code]: https://www.40code.com/
[gitblock]: https://gitblock.cn