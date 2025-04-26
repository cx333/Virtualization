#!/bin/ash

# 安装 openssh 并配置
apk add openssh
echo "PermitRootLogin yes" >> /etc/ssh/sshd_config
echo "root:alpine" | chpasswd
rc-update add sshd default
rc-service sshd start

# ===== 网络配置（静态 IP） =====
# 注意 eth0 是虚拟机里的网卡名
# IP_ADDR=${ip:-192.168.1.100}  # 读取内核参数传入的 IP 地址（如果有多个虚拟机）
ip addr add 192.168.1.100/24 dev eth0
ip link set eth0 up
ip route add default via 192.168.1.1
