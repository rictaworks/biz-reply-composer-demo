import { describe, it, expect } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MainPage } from "./MainPage";
import "@/i18n";

// Tauri外（vitest）ではモックバックエンドが使われる。
describe("MainPage", () => {
  it("本文が10文字未満のとき生成ボタンは無効", async () => {
    render(<MainPage />);
    const button = await screen.findByRole("button", { name: /返信文を生成/ });
    expect(button).toBeDisabled();

    await userEvent.type(screen.getByLabelText(/受信メール本文/), "短い");
    expect(button).toBeDisabled();
  });

  it("10文字以上入力すると生成でき、4部構成トークン付きドラフトが表示される", async () => {
    render(<MainPage />);
    const textarea = await screen.findByLabelText(/受信メール本文/);
    await userEvent.type(
      textarea,
      "お世話になっております。来週の打ち合わせ日程についてご相談です。",
    );

    const button = screen.getByRole("button", { name: /返信文を生成/ });
    await waitFor(() => expect(button).toBeEnabled());
    await userEvent.click(button);

    // 宛名・署名トークンが固定挿入されている（固有名詞捏造禁止）。
    expect(await screen.findByText(/生成された返信文ドラフト/)).toBeInTheDocument();
    const body = screen.getByText(/\{\{相手名\}\}/);
    expect(body.textContent).toContain("{{自分の署名}}");
  });
});
