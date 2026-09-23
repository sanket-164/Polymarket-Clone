import { OrderDetailsPage } from "@/components/order/OrderDetailsPage";

export default async function OrderDetailsRoutePage({
  params,
}: {
  params: Promise<{ orderId: string }>;
}) {
  const { orderId } = await params;

  return <OrderDetailsPage orderId={orderId} />;
}
