import { Navigate, NavLink, Route, Routes } from "react-router-dom";
import { MarketsPage } from "./pages/MarketsPage";
import { MarketDetailPage } from "./pages/MarketDetailPage";

function App() {
  return (
    <Routes>
      <Route path="/" element={<Navigate to="/markets" replace />} />
      <Route path="/markets" element={<MarketsPage />} />
      <Route path="/markets/:marketId" element={<MarketDetailPage />} />
      <Route
        path="*"
        element={
          <main className="grid min-h-screen place-items-center bg-slate-950 px-6 text-white">
            <div className="text-center">
              <p className="text-sm uppercase tracking-[0.3em] text-slate-400">
                404
              </p>
              <h1 className="mt-3 text-3xl font-bold">Page not found</h1>
              <NavLink
                to="/markets"
                className="mt-6 inline-flex rounded-full bg-cyan-300 px-5 py-2.5 text-sm font-semibold text-slate-950"
              >
                Back to markets
              </NavLink>
            </div>
          </main>
        }
      />
    </Routes>
  );
}

export default App;
