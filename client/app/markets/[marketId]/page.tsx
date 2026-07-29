import { MarketDetailsPage } from "@/components/market/MarketDetailsPage";

type MarketPageProps = {
  params: Promise<{
    marketId: string;
  }>;
};

export default async function MarketPage({ params }: MarketPageProps) {
  const { marketId } = await params;

  return <MarketDetailsPage marketId={marketId} />;
}
