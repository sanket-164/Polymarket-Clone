export function AuthCard({
  eyebrow,
  title,
  children,
}: {
  eyebrow: string;
  title: string;
  children: React.ReactNode;
}) {
  return (
    <section className="mx-auto w-full max-w-md px-4 py-8 sm:px-6">
      <div className="rounded-2xl border border-border bg-surface p-6">
        <p className="text-sm text-secondary">{eyebrow}</p>
        <h1 className="mt-2 text-3xl font-bold text-text">{title}</h1>
        {children}
      </div>
    </section>
  );
}
