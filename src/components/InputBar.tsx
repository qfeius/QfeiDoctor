import { useState } from "react";

interface InputBarProps {
  onStart: (target: string) => void;
  onCopy: () => void;
  isRunning: boolean;
  hasResult: boolean;
}

export function InputBar({
  onStart,
  onCopy,
  isRunning,
  hasResult,
}: InputBarProps) {
  const [target, setTarget] = useState("https://contract.qfei.cn");

  const handleStart = () => {
    const trimmed = target.trim();
    if (trimmed) {
      onStart(trimmed);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !isRunning) {
      handleStart();
    }
  };

  return (
    <div className="input-bar">
      <input
        className="input-bar__field"
        type="text"
        placeholder="输入 URL 或域名，如 https://example.com 或 example.com"
        value={target}
        onChange={(e) => setTarget(e.target.value)}
        onKeyDown={handleKeyDown}
        disabled={isRunning}
      />
      <button
        className="btn btn--primary"
        onClick={handleStart}
        disabled={isRunning || !target.trim()}
      >
        {isRunning ? "分析中..." : "开始分析"}
      </button>
      {hasResult && (
        <button className="btn btn--outline" onClick={onCopy}>
          复制反馈信息
        </button>
      )}
    </div>
  );
}
