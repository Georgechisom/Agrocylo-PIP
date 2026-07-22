import { Route, Routes } from 'react-router-dom';
import DesignFoundationsPage from './pages/DesignFoundationsPage';
import { AnalyticsDashboardPage } from './pages/AnalyticsDashboardPage';
import { InvestorDashboardPage } from './pages/InvestorDashboardPage';

function App() {
  return (
    <Routes>
      <Route path="/" element={<DesignFoundationsPage />} />
      <Route path="/analytics" element={<AnalyticsDashboardPage />} />
      <Route path="/investor" element={<InvestorDashboardPage />} />
    </Routes>
  );
}

export default App;
