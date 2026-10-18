export type ToastMessage = {
  text: string;
  action?: { label: string; run: () => void };
};

export function Toast(props: { message: ToastMessage; onDismiss: () => void }) {
  const { text, action } = props.message;

  return (
    <div class="toast shh" role="status">
      <span>{text}</span>
      {action && (
        <button
          type="button"
          onClick={() => {
            action.run();
            props.onDismiss();
          }}
        >
          {action.label}
        </button>
      )}
    </div>
  );
}
