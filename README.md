# rust-s3c2440

科技考古：使用Rust开发三星S3C2440 SoC驱动的TQ2440开发板。

北京邮电大学研究生嵌入式技术与实验课程成果仓库。

### 基本信息

三星S3C2440 SoC是三星推出的基于ARM920T内核设计的32位RISC控制器，主频可达400MHz，提供了丰富的外设接口。

S3C2440使用的ARM指令集架构为ARMv4T，这也是Rust编译器（或者说LLVM编译器）支持的最老一代ARM指令集架构，名称为`armv4t-none-eabi`，详解[rustc手册](https://doc.rust-lang.org/rustc/platform-support/armv4t-none-eabi.html)。

> Fun Fact: 这也是任天堂推出的著名游戏机Game Boy Advance使用的指令集架构。

为了压缩生成的可执行文件体积，该CPU还支持`thumb`模式，即16位的短指令长度。

### 代码结构

本项目中使用硬件抽象层（Hardware Abstraction Layer，HAL）、标准库（Standard Library，STD）和应用程序（Application，APP）的三级抽象结构。

硬件抽象层`libraries/s3c2440-hal`提供了Rust封装的针对各种硬件、外设的访问、控制原语，编写的过程中尽可能保证使用平台无关的指令，使得该库编写的单元测试可以直接在amd64的台式机上运行。具体的，本硬件抽象层提供了对于如下硬件的抽象：

- GPIO接口，提供了基于自动机的GPIO端口配置和输入输出模块，并实现了对应的`embedded-hal`trait。
- Clock接口，提供了控制芯片内各时钟启停的方法。
- DMA接口，提供了控制芯片内提供的4个DMA引擎的相关配置和使用操作。
- IIS接口，提供了控制芯片内IIS控制器相关的操作。
- Interrupt接口，提供了芯片内中断控制器相关的操作。
- L3BUS接口，提供了控制板上UDA11341音频编解码芯片的能力。
- NAND接口，提供了编程读写板上S34ML02G1闪存芯片的能力。
- UART接口，提供了使用芯片UART接口进行输入输出的能力。

标准库`libraries/s3c2440-std`则起到了一个简易操作系统的作用，封装了系统引导、中断处理，堆内存初始化等功能，并提供了一系列便于使用的函数，例如输入输出，中断的注册和删除和播放音乐等。

应用程序则是一个完成具体功能的具体个体，具体列举如下：

- `applications/uart-test`：仅使用最基本的引导程序，测试UART部件的初始化和输出是否正常。
- `applications/flash-test`：读写NAND的基本引导程序。
- `applications/interrupt-test`：测试中断的注册、触发、处理、取消注册是否正常。
- `applications/audio-test`：播放音乐的测试程序。
- `applications/experiment`：最终的实验验收程序，实现了所有三个实验验收功能，包括UART输入输出、NAND闪存读写和音乐播放等。

### 编译与测试

因为`armv4t-none-eabi`属于Rust支持的Tier 3目标，因此没有提供预编译的`core`或者`std`包，需要使用`build-std`自行构建，因此只能使用`nightly`版本的编译器编译运行：

```bash
rustup toolchain install nightly
```

> 不需要手动切换为`nightly`版本，项目中已使用`rust-toolchain.toml`和`config.toml`配置文件设置合适的编译工具链和编译参数。

编译得到的二进制可执行文件位于`target/armv4t-none-eabi/debug`文件夹中，实验中仅验证了使用老师提供的bootloader加载FTP挂载的二进制文件启动的正确性（即热启动），没有测试将二进制文件烧写到Flash中启动的可行性（即冷启动）。

具体地说，板子上的bootloader在启动之后会从指定地址的FTP服务器上下载名为`vxWorks.tq2240`的ELF文件，并加载到`0x30001000`开始执行，因此只需要替换对应的文件就可以实现执行自行编写的应用程序。

这里推荐使用`sftpgo`来启动一个便携的FTP服务器，指令如下：

```bash
sftpgo portable -u target -p 9 --ftpd-port 21
```

### 支持

如果您在学习或者是抄袭的过程中发现了问题，我们十分欢迎您提出，您可以通过发起`issue`或者是发送电子邮件的方式联系我们。