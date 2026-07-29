type FormFieldProps = {
  id: string;
  label: string;
  type?: string;
  name: string;
  autoComplete?: string;
  required?: boolean;
  minLength?: number;
};

export function FormField({
  id,
  label,
  type = "text",
  name,
  autoComplete,
  required = true,
  minLength,
}: FormFieldProps) {
  return (
    <label htmlFor={id} className="block">
      <span className="text-sm font-medium text-text">{label}</span>
      <input
        id={id}
        name={name}
        type={type}
        autoComplete={autoComplete}
        required={required}
        minLength={minLength}
        className="mt-2 h-11 w-full rounded-lg border border-border bg-card px-3 text-sm text-text outline-none transition placeholder:text-secondary focus:border-accent focus:ring-2 focus:ring-accent/25"
      />
    </label>
  );
}
