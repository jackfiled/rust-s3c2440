#!/usr/bin/env python3
import sys
import wave
import struct
import argparse

def convert_wav_to_bigendian_samples(input_path: str, output_path: str):
    with wave.open(input_path, 'rb') as wf_in:
        n_channels = wf_in.getnchannels()
        samp_width = wf_in.getsampwidth()
        frame_rate = wf_in.getframerate()
        n_frames = wf_in.getnframes()

        print(f"原始 WAV 信息:")
        print(f"  声道数: {n_channels}")
        print(f"  位深: {samp_width * 8}-bit")
        print(f"  采样率: {frame_rate} Hz")
        print(f"  总帧数: {n_frames}")

        if samp_width != 2:
            raise ValueError("仅支持 16-bit (2 bytes per sample) WAV 文件")

        # 读取所有音频帧（字节流）
        raw_frames = wf_in.readframes(n_frames)

    # 计算总样本数（每个样本 2 字节）
    total_samples = len(raw_frames) // 2

    # 将字节流解包为小端 i16 列表（验证用，可选）
    # samples_le = struct.unpack(f"<{total_samples}h", raw_frames)

    # 🔥 关键：将每个 16-bit 样本从小端转为大端字节序
    # 方法：每 2 字节一组，交换顺序
    be_frames = bytearray()
    for i in range(0, len(raw_frames), 2):
        # raw_frames[i] = low byte, raw_frames[i+1] = high byte (little-endian)
        # 大端顺序：high byte first, then low byte
        be_frames.append(raw_frames[i + 1])
        be_frames.append(raw_frames[i])

    # 写入新 WAV 文件（头仍为小端，样本为大端）
    with wave.open(output_path, 'wb') as wf_out:
        wf_out.setnchannels(n_channels)
        wf_out.setsampwidth(samp_width)
        wf_out.setframerate(frame_rate)
        wf_out.writeframes(be_frames)

    print(f"\n✅ 转换完成!")
    print(f"  输入: {input_path}")
    print(f"  输出: {output_path}")
    print(f"  说明: 文件头保持小端（标准），仅音频样本转为大端字节序")
    print(f"  ⚠️  此文件不符合标准 WAV 规范，仅用于特定硬件测试（如 S3C2440）")

def main():
    parser = argparse.ArgumentParser(
        description="将标准 WAV 文件的音频样本转换为大端字节序（文件头保持小端）"
    )
    parser.add_argument("input", help="输入的标准小端 WAV 文件")
    parser.add_argument("output", help="输出的大端样本 WAV 文件")
    args = parser.parse_args()

    try:
        convert_wav_to_bigendian_samples(args.input, args.output)
    except Exception as e:
        print(f"❌ 错误: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
