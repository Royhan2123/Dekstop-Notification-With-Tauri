// @vitest-environment jsdom
import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";
import App from "./App";

it("menampilkan nama aplikasi", () => {
  render(<App />);
  expect(screen.getByText("Isla Desk")).toBeTruthy();
});
