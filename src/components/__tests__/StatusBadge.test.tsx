import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { StatusBadge } from "../StatusBadge";

describe("StatusBadge", () => {
  it("renders pass status", () => {
    render(<StatusBadge status="pass" />);
    expect(screen.getByText("通过")).toBeDefined();
  });

  it("renders fail status", () => {
    render(<StatusBadge status="fail" />);
    expect(screen.getByText("失败")).toBeDefined();
  });

  it("applies correct CSS class", () => {
    const { container } = render(<StatusBadge status="warn" />);
    const badge = container.querySelector(".status-badge--warn");
    expect(badge).not.toBeNull();
  });
});
