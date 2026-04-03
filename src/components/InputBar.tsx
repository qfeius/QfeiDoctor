import { useRef, useEffect } from "react";

interface InputBarProps {
  target: string;
  onTargetChange: (value: string) => void;
  onStart: () => void;
  onCopy: () => void;
  isRunning: boolean;
  hasResult: boolean;
}

export function InputBar({
  target,
  onTargetChange,
  onStart,
  onCopy,
  isRunning,
  hasResult,
}: InputBarProps) {
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !isRunning) {
      onStart();
    }
  };

  return (
    <div className="input-bar">
      <input
        ref={inputRef}
        className="input-bar__field"
        type="text"
        placeholder="输入 URL，如 https://contract.qfei.cn"
        value={target}
        onChange={(e) => onTargetChange(e.target.value)}
        onKeyDown={handleKeyDown}
        disabled={isRunning}
      />
      <button
        className="btn btn--primary"
        onClick={onStart}
        disabled={isRunning || !target.trim()}
      >
        {isRunning ? "分析中..." : "开始分析"}
      </button>
      <button
        className="btn btn--outline"
        onClick={onCopy}
        disabled={!hasResult}
      >
        复制反馈信息
      </button>
    </div>
  );
}
