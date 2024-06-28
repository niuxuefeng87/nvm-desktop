import { Closer, Themes } from 'types';
// import type { UpdateInfo as ElectronUpdateInfo } from "electron-updater";

type Platform =
	| 'aix'
	| 'android'
	| 'darwin'
	| 'freebsd'
	| 'haiku'
	| 'linux'
	| 'openbsd'
	| 'sunos'
	| 'win32'
	| 'cygwin'
	| 'netbsd';
type Architecture =
	| 'arm'
	| 'arm64'
	| 'ia32'
	| 'loong64'
	| 'mips'
	| 'mipsel'
	| 'ppc'
	| 'ppc64'
	| 'riscv64'
	| 's390'
	| 's390x'
	| 'x64';

declare global {
	/**
	 * defines in `vite.config.ts`
	 */
	declare const OS_ARCH: Architecture;
	declare const OS_PLATFORM: Platform;

	namespace Nvmd {
		interface Version {
			version: string;
			npm: string;
			lts: string | false;
			date: string;
			v8: string;
			files: string[];
		}

		type Versions = Array<Version>;

		interface ProgressData {
			percent: number;
			transferred: number;
			total: number;
		}

		interface Setting {
			locale: string;
			theme: Themes;
			closer: Closer;
			directory: string;
			mirror: string;
		}

		// type UpdateInfo = ElectronUpdateInfo | "update-not-available";

		interface Project {
			name: string;
			path: string;
			version?: string;
			active: boolean;
			createAt: string;
			updateAt: string;
		}

		interface Group {
			name: string;
			desc?: string;
			version: string;
			projects: string[];
		}

		interface ConfigrationExport {
			color?: string;
			mirrors?: string | null;
			path: string;
			projects?: boolean;
			setting?: boolean;
		}

		interface Configration {
			color?: string;
			mirrors?: string;
			setting?: Setting;
			projects?: Project[];
			groups?: Group[];
		}
	}
}
