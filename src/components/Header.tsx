import { Stethoscope } from "lucide-react";

export function Header() {
  return (
    <header className="header">
      <h1>
        <Stethoscope size={24} style={{ verticalAlign: "middle", marginRight: 8 }} />
        智书诊断助手
      </h1>
    </header>
  );
}
