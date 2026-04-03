export function Header() {
  return (
    <header className="header">
      <h1>
        <img
          src="/favicon.png"
          alt="Logo"
          width={28}
          height={28}
          style={{ verticalAlign: "middle", marginRight: 8, borderRadius: 6 }}
        />
        智书诊断助手
      </h1>
    </header>
  );
}
