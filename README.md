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
|**[CCW]**|:white_check_mark:||
|**[Clipcc]**|:white_check_mark:||
|**[小码王][xmw]**|:white_check_mark:||
|**[Scratch 中社][scratch-cn]**|:white_check_mark:|从 `v0.2.1` 开始支持|
|**[40code]**|:construction:|有 40code 作品 **只支持 Firefox 的 `User-Agent` 访问**, 暂时仍未解决|

<dd>
:white_check_mark: 全部支持

:construction: 支持但仍存在问题
</dd>

### :package:安装

[![ci](https://github.com/LycasLdt/rabdog/actions/workflows/ci.yml/badge.svg)](https://github.com/LycasLdt/rabdog/actions/workflows/ci.yml)

在 [Releases][download] 中下载

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

[ccw]: https://www.ccw.site
[clipcc]: https://codingclip.com
[40code]: https://40code.com
[xmw]: https://world.xiaomawang.com/
[scratch-cn]: https://www.scratch-cn.cn/
[40code]: https://www.40code.com/