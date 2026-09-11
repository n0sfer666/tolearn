interface Props {
  reason: string;
  locale: string;
  label: string;
}

export default function Empty(props: Props) {
  return (
    <p data-empty>
      {props.reason} <a href={`/${props.locale}/`}>{props.label}</a>
    </p>
  );
}
