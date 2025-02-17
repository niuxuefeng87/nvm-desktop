import './global.css';

import { useEffect } from 'react';
import { RouterProvider } from 'react-router';
import { Toaster, TooltipProvider } from '@/components/ui';

import { router } from '@/routes';
import { AppProvider } from '@/app-context';
import { getCurrent } from '@/services/api';
import { SystemTheme } from '@/types';

export default function App({
	settings,
	sysTheme,
}: {
	settings: Nvmd.Setting;
	sysTheme: SystemTheme;
}) {
	useEffect(() => {
		// open main webview window
		setTimeout(() => {
			const webviewWindow = getCurrent();
			webviewWindow.show();
			webviewWindow.setFocus();
		});
	}, []);

	/// Disable right-click context menu
	useEffect(() => {
		const handleContextMenu = (e: MouseEvent) => {
			e.preventDefault();
		};
		document.addEventListener('contextmenu', handleContextMenu);
		return () => {
			document.removeEventListener('contextmenu', handleContextMenu);
		};
	}, []);

	return (
		<AppProvider settings={settings} sysTheme={sysTheme}>
			<TooltipProvider delayDuration={200}>
				<RouterProvider router={router} />
			</TooltipProvider>
			<Toaster />
		</AppProvider>
	);
}
