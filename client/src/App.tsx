import { Route, Routes } from 'react-router-dom';
import DesignFoundationsPage from './pages/DesignFoundationsPage';
import { AnalyticsDashboardPage } from './pages/AnalyticsDashboardPage';
import { CampaignsPage, CampaignDetailPage, CreateCampaignPage } from './pages';

function App() {
  return (
    <Routes>
      <Route path="/" element={<DesignFoundationsPage />} />
      <Route path="/analytics" element={<AnalyticsDashboardPage />} />
      <Route path="/campaigns" element={<CampaignsPage />} />
      <Route path="/campaigns/new" element={<CreateCampaignPage />} />
      <Route path="/campaigns/:campaignId" element={<CampaignDetailPage />} />
    </Routes>
  );
}

export default App;
