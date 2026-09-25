/**
 * Closes only when the press and release both land on the backdrop. A plain
 * click.self also fires when a text-selection drag starts inside the dialog
 * and ends on the backdrop.
 */
export function useBackdropClose(close: () => void) {
  let pressedBackdrop = false;

  function onPointerdown(event: PointerEvent) {
    pressedBackdrop = event.target === event.currentTarget;
  }

  function onClick(event: MouseEvent) {
    if (pressedBackdrop && event.target === event.currentTarget) {
      close();
    }
    pressedBackdrop = false;
  }

  return { onPointerdown, onClick };
}
