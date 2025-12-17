import math
import struct
import wave

# 配置参数
SAMPLE_RATE = 44100      # 采样率 (Hz)
DURATION_ON = 1.0        # 音频持续时间 (秒)
DURATION_OFF = 1.0       # 静音持续时间 (秒)
FREQ = 440               # 音调频率 (Hz)
REPEAT = 3               # 重复次数
CHANNELS = 2             # 立体声
BYTES_PER_SAMPLE = 2     # 16-bit = 2 bytes

def generate_sine_wave(freq, duration, sample_rate):
    """生成单声道正弦波样本（i16 列表）"""
    num_samples = int(duration * sample_rate)
    samples = []
    for i in range(num_samples):
        t = i / sample_rate
        value = math.sin(2.0 * math.pi * freq * t)
        # 转为 16-bit 有符号整数
        sample = int(value * 32767)
        # 限幅（虽然正弦不会超，但安全起见）
        sample = max(-32768, min(32767, sample))
        samples.append(sample)
    return samples

def generate_silence(duration, sample_rate):
    """生成静音（全 0）"""
    num_samples = int(duration * sample_rate)
    return [0] * num_samples

# 构建完整音频帧（交错立体声）
frames = []
for _ in range(REPEAT):
    # 1 秒音
    sine = generate_sine_wave(FREQ, DURATION_ON, SAMPLE_RATE)
    # 1 秒静音
    silence = generate_silence(DURATION_OFF, SAMPLE_RATE)
    # 合并单声道为立体声（L=R）
    for s in sine + silence:
        frames.append(s)  # Left
        frames.append(s)  # Right

name = "beep_silence_22050_stereo_big_endian.wav"

# 写入 WAV 文件
with wave.open(name, "wb") as wf:
    wf.setnchannels(CHANNELS)
    wf.setsampwidth(BYTES_PER_SAMPLE)
    wf.setframerate(SAMPLE_RATE)
    # 将 i16 列表转为小端字节流
    raw_data = struct.pack("<{}h".format(len(frames)), *frames)
    wf.writeframes(raw_data)  # raw 是 bytes

print(f"✅ 已生成: {name}")
print(f"   采样率: {SAMPLE_RATE} Hz, 位深: 16-bit, 声道: 立体声")
print(f"   内容: {REPEAT} 次 (1s 440Hz + 1s 静音)")
