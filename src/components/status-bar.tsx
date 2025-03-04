export function StatusBar(props: { text: string }) {
  return (
    <div
      style={{
        position: "absolute",
        bottom: 0,
        left: 0,
        right: 0,
        height: "20px",
        backgroundColor: "rgba(255, 255, 255, 0.1)",
        backdropFilter: "blur(10px)",
        WebkitBackdropFilter: "blur(10px)",
        padding: "0 10px",
        display: "flex",
        alignItems: "center",
        fontSize: "12px",
      }}
    >
      Chars: {props.text.length}
    </div>
  );
}
