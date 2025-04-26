use std::process::{Command, Stdio};
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("用法: vm [create|start|stop|destroy]");
        return;
    }

    match args[1].as_str() {
        "create" => create_vm(),
        "start" => start_vm(),
        "stop" => stop_vm(),
        "destroy" => destroy_vm(),
        _ => println!("未知命令: {}", args[1]),
    }
}

fn create_vm() {
    let vm_path = "vm/alpine_disk.img";
    
    if Path::new(vm_path).exists() {
        println!("⚠️ 虚拟机磁盘文件已存在");
        return;
    }

    println!("🛠️ 创建虚拟机...");

    // 创建虚拟机磁盘文件
    Command::new("qemu-img")
        .args(["create", "-f", "qcow2", vm_path, "2G"])
        .status()
        .expect("❌ 创建虚拟机磁盘失败");

    println!("✅ 虚拟机创建完成，磁盘文件: {}", vm_path);
}

fn start_vm() {
    let pid_path = format!("vm/alpine.pid");

    if Path::new("vm/alpine.pid").exists() {
        println!("⚠️ 虚拟机已在运行");
        return;
    }

    let vm_disk_path = format!("vm/alpine_disk.img");

    if !Path::new("vm/alpine_disk.img").exists() {
        println!("❌ 虚拟机磁盘文件不存在，请先创建虚拟机");
        return;
    }

    println!("🚀 启动 Alpine 虚拟机...");

    let child = Command::new("qemu-system-aarch64")
    .args([
        "-machine", "virt,accel=kvm",
        "-cpu", "host",
        "-m", "512",
        "-smp", "2",
        "-bios", "/usr/share/qemu-efi-aarch64/QEMU_EFI.fd",
        "-cdrom", "vm-images/alpine.iso",
        "-drive", &format!("file={},if=virtio,format=qcow2", vm_disk_path),
        // "-nic", "user,model=virtio-net-pci,hostfwd=tcp::2222-:22",
        "-netdev", "tap,id=net0,ifname=tap0,script=no,downscript=no", // 使用 tap 设备
        "-device", "virtio-net-device,netdev=net0", // 将 tap 设备与 virtio 网络适配器关联
        "-initrd", "vm-images/custom-initramfs.gz", // 包含配置脚本的initramfs
        "-kernel", "vm-images/vmlinuz-lts",
        "-initrd", "vm-images/initramfs-lts",
        "-append", "console=ttyAMA0,115200 console=ttyS0",
        "-nographic",
        "-serial", "mon:stdio",  // 关键修改
        "-monitor", "none",      // 关键修改
    ])
    .stdin(Stdio::inherit())  // 继承输入以便交互
    .stdout(Stdio::inherit()) // 直接输出到终端
    .stderr(Stdio::inherit())
    .spawn()
    .expect("❌ 启动失败");

    fs::write("vm/alpine.pid", child.id().to_string()).unwrap();
    fs::write(&pid_path, child.id().to_string()).unwrap();
    println!("✅ 虚拟机启动完成，PID: {}", child.id());
    let output = child
    .wait_with_output()
    .expect("❌ 获取输出失败");

println!("QEMU 输出: {:?}", output);
}

fn stop_vm() {
    let pid_path = "vm/alpine.pid";
    
    if let Ok(pid_str) = fs::read_to_string(pid_path) {
        let pid = pid_str.trim();
        println!("🛑 停止虚拟机，PID: {}", pid);
        
        // 检查进程是否仍然存在
        let process_exists = Command::new("ps")
            .args(["-p", pid])
            .output()
            .expect("❌ 检查进程失败")
            .stdout;
        
        if !process_exists.is_empty() {
            // 如果进程存在，则终止它
            Command::new("kill")
                .arg(pid)
                .status()
                .expect("❌ 停止虚拟机失败");
            fs::remove_file(pid_path).ok();
            println!("✅ 已停止");
        } else {
            println!("❌ 找不到运行中的虚拟机进程");
        }
    } else {
        println!("❌ 没有运行中的虚拟机");
    }
}

fn destroy_vm() {
    stop_vm(); // 先停止
    fs::remove_file("vm/alpine_disk.img").ok();
    fs::remove_file("vm/alpine.pid").ok();
    println!("💥 虚拟机已销毁（磁盘删除）");
}
