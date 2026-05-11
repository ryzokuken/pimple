import { fireEvent, render } from "@testing-library/svelte";
import { describe, expect, test, vi } from "vitest";

import RecurringScopeDialog from "../src/lib/components/RecurringScopeDialog.svelte";

describe("RecurringScopeDialog", () => {
  test("each button fires onChoose with the corresponding RecurringScope value", async () => {
    const onChoose = vi.fn();
    const onCancel = vi.fn();
    const { getByTestId } = render(RecurringScopeDialog, {
      verb: "delete",
      onChoose,
      onCancel,
    });

    await fireEvent.click(getByTestId("scope-this-instance"));
    expect(onChoose).toHaveBeenLastCalledWith("this_instance");

    await fireEvent.click(getByTestId("scope-this-and-future"));
    expect(onChoose).toHaveBeenLastCalledWith("this_and_future");

    await fireEvent.click(getByTestId("scope-all"));
    expect(onChoose).toHaveBeenLastCalledWith("all");

    expect(onCancel).not.toHaveBeenCalled();
  });

  test("Escape key triggers onCancel", async () => {
    const onChoose = vi.fn();
    const onCancel = vi.fn();
    render(RecurringScopeDialog, { verb: "delete", onChoose, onCancel });

    await fireEvent.keyDown(window, { key: "Escape" });
    expect(onCancel).toHaveBeenCalledOnce();
    expect(onChoose).not.toHaveBeenCalled();
  });

  test("verb='edit' uses edit-flavoured copy", () => {
    const onChoose = vi.fn();
    const onCancel = vi.fn();
    const { getByText } = render(RecurringScopeDialog, {
      verb: "edit",
      onChoose,
      onCancel,
    });
    expect(getByText(/edit recurring event/i)).toBeTruthy();
  });
});
