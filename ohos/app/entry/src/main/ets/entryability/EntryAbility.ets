import fs from "@ohos.file.fs";
import hilog from "@ohos.hilog";
import wantAgent from "@ohos.app.ability.wantAgent";
import { UrlPlugin } from "@ohos-rs/ability-plugin-url";
import { MenuPlugin } from "@ohos-rs/ability-plugin-menu";
import { FilesPlugin } from "@ohos-rs/ability-plugin-files";
import { LazyPlugin, NativeAbility } from "@ohos-rs/ability";
import { WindowPlugin } from "@ohos-rs/ability-plugin-window";
import { backgroundTaskManager } from "@kit.BackgroundTasksKit";
import { ClipboardPlugin } from "@ohos-rs/ability-plugin-clipboard";
import { PermissionPlugin } from "@ohos-rs/ability-plugin-permission";
import { AppControlPlugin } from "@ohos-rs/ability-plugin-app-control";
import { SystemStatePlugin } from "@ohos-rs/ability-plugin-system-state";
import { NotificationPlugin } from "@ohos-rs/ability-plugin-notification";

const DOMAIN: number = 0xff00;
const TAG: string = "mtorrent-bg";
const ACTIVITY_FILE: string = "bg_active";
const DATA_TRANSFER_MODES: string[] = ["dataTransfer"];

export default class EntryAbility extends NativeAbility {
  public moduleName: string = "mtorrent_gpui_ohos";
  public defaultPage: boolean = true;
  public bridgePlugins = [
    new LazyPlugin(() => new UrlPlugin()),
    new LazyPlugin(() => new MenuPlugin()),
    new LazyPlugin(() => new FilesPlugin()),
    new LazyPlugin(() => new WindowPlugin()),
    new LazyPlugin(() => new ClipboardPlugin()),
    new LazyPlugin(() => new PermissionPlugin()),
    new LazyPlugin(() => new AppControlPlugin()),
    new LazyPlugin(() => new SystemStatePlugin()),
    new LazyPlugin(() => new NotificationPlugin({ scheme: "mtorrent-notification" })),  ];

  onBackground() {
    super.onBackground();
    if (this.isDownloading())
      this.requestDataTransfer();
  }

  onForeground() {
    super.onForeground();
    this.releaseDataTransfer();
  }

  private isDownloading(): boolean {
    try {
      const path = `${this.context.filesDir}/${ACTIVITY_FILE}`;
      if (!fs.accessSync(path))
        return false;
      const file = fs.openSync(path, fs.OpenMode.READ_ONLY);
      try {
        const buffer = new ArrayBuffer(1);
        const length = fs.readSync(file.fd, buffer);
        const bytes = new Uint8Array(buffer, 0, length);
        return length > 0 && bytes[0] === 0x31;
      } finally {
        fs.closeSync(file);
      }
    } catch (error) {
      const err = error as BusinessError;
      hilog.error(DOMAIN, TAG, "Read activity marker failed: %{public}s", err.message ?? "");
      return false;
    }
  }

  private requestDataTransfer() {
    try {
      wantAgent.getWantAgent({
        wants: [
          {
            abilityName: this.context.abilityInfo.name,
            bundleName: this.context.abilityInfo.bundleName,
          },
        ],
        actionType: wantAgent.OperationType.START_ABILITY,
        requestCode: 0,
        actionFlags: [wantAgent.WantAgentFlags.UPDATE_PRESENT_FLAG],
      }).then((agent) => {
        try {
          backgroundTaskManager.startBackgroundRunning(this.context, DATA_TRANSFER_MODES, agent).then((result) => {
            hilog.info(DOMAIN, TAG, "Continuous task started, notificationId %{public}d", result.notificationId,);
          }).catch((error: BusinessError) => {
            hilog.error(DOMAIN, TAG, "Start continuous task failed, code %{public}d, message %{public}s", error.code,
              error.message ?? "");
          });
        } catch (error) {
          const err = error as BusinessError;
          hilog.error(DOMAIN, TAG, "Start continuous task threw: %{public}s", err.message ?? "");
        }
      }).catch((error: BusinessError) => {
        hilog.error(DOMAIN, TAG, "Get want agent failed, code %{public}d, message %{public}s",
          error.code, error.message ?? "");
      });
    } catch (error) {
      const err = error as BusinessError;
      hilog.error(DOMAIN, TAG, "Get want agent threw: %{public}s", err.message ?? "");
    }
  }

  private releaseDataTransfer() {
    try {
      backgroundTaskManager.stopBackgroundRunning(this.context).then(() => {
        hilog.info(DOMAIN, TAG, "Continuous task stopped.");
      }).catch((error: BusinessError) => {
        hilog.error(DOMAIN, TAG, "Stop continuous task failed, code %{public}d, message %{public}s", error.code,
          error.message ?? "");
      });
    } catch (error) {
      const err = error as BusinessError;
      hilog.error(DOMAIN, TAG, "Stop continuous task threw: %{public}s", err.message ?? "");
    }
  }
}