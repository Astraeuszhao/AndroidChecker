#!/bin/bash
# 测试备份功能的脚本

echo "=== Android Backup Test Script ==="
echo ""

# 设置路径
SOURCE_DIR="E:/test-android-backup/mock-device/sdcard"
BACKUP_DIR="E:/test-android-backup/backups"
RESTORE_DIR="E:/test-android-backup/restore-test"

# 清理之前的测试
rm -rf "$BACKUP_DIR"/*.adbbackup
rm -rf "$RESTORE_DIR"/*

echo "1. 测试备份打包..."
cd "$SOURCE_DIR"
BACKUP_FILE="$BACKUP_DIR/test_device_$(date +%Y%m%d_%H%M%S).adbbackup"

# 创建 tar.gz 并重命名为 .adbbackup
tar -czf "$BACKUP_FILE" ./*

if [ -f "$BACKUP_FILE" ]; then
    echo "   ✓ 备份文件创建成功: $BACKUP_FILE"
    echo "   大小: $(du -h "$BACKUP_FILE" | cut -f1)"
else
    echo "   ✗ 备份文件创建失败"
    exit 1
fi

echo ""
echo "2. 测试备份解压..."
cd "$RESTORE_DIR"
tar -xzf "$BACKUP_FILE"

if [ $? -eq 0 ]; then
    echo "   ✓ 备份文件解压成功"
else
    echo "   ✗ 备份文件解压失败"
    exit 1
fi

echo ""
echo "3. 验证文件完整性..."
diff -r "$SOURCE_DIR" "$RESTORE_DIR"

if [ $? -eq 0 ]; then
    echo "   ✓ 所有文件完全一致！"
else
    echo "   ⚠ 文件有差异，请检查"
fi

echo ""
echo "4. 列出恢复的文件："
ls -lR "$RESTORE_DIR"

echo ""
echo "=== 测试完成 ==="
