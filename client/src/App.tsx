import { createBrowserRouter, RouterProvider } from 'react-router-dom'
import { AppLayout } from './components/AppLayout'
import {
  ActivityFeedPage,
  AdminDashboardPage,
  CampaignDetailPage,
  CampaignsPage,
  CreateCampaignPage,
  FarmerDashboardPage,
  HomePage,
  InvestorDashboardPage,
  NotFoundPage,
  ProfilePage,
} from './pages'

const router = createBrowserRouter([
  {
    path: '/',
    element: <AppLayout />,
    children: [
      { index: true, element: <HomePage /> },
      { path: 'campaigns', element: <CampaignsPage /> },
      { path: 'campaigns/new', element: <CreateCampaignPage /> },
      { path: 'campaigns/:campaignId', element: <CampaignDetailPage /> },
      { path: 'dashboard/farmer', element: <FarmerDashboardPage /> },
      { path: 'dashboard/investor', element: <InvestorDashboardPage /> },
      { path: 'dashboard/admin', element: <AdminDashboardPage /> },
      { path: 'activity', element: <ActivityFeedPage /> },
      { path: 'profile', element: <ProfilePage /> },
      { path: '*', element: <NotFoundPage /> },
    ],
  },
])

function App() {
  return <RouterProvider router={router} />
}

export default App
