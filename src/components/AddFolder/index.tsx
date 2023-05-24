import React, { useEffect, useRef, useState, ChangeEvent } from "react";
import { useModal } from "../Modal/useModal";
import * as dataAgent from "../../helpers/dataAgent";
import { busChannel } from "../../helpers/busChannel";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Icon } from "../Icon";
import { Folder as Folder2, Loader2 } from "lucide-react";
import { Button } from "../ui/button";
import { Channel } from "@/db";

export interface AddFolderProps {
  action: "add" | "edit";
  folder?: Channel | null;
  dialogStatus: boolean;
  trigger?: React.ReactNode;
  setDialogStatus: (status: boolean) => void;
  afterConfirm: () => void;
  afterCancel: () => void;
}

export const AddFolder = (props: AddFolderProps) => {
  const { action, folder } = props;
  const { dialogStatus, setDialogStatus, afterConfirm, afterCancel, trigger } =
    props;
  const [name, setName] = useState("");
  const [confirming, setConfirming] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  const handleNameChange = (value: string) => {
    setName(value);
  };

  const handleCancel = () => {
    setConfirming(false);
    setName("");
    setDialogStatus(false);
    afterCancel();
  };

  const handleSave = async () => {
    setConfirming(true);

    let p: Promise<any> = Promise.resolve();

    if (action === "add") {
      p = dataAgent.createFolder(name);
    } else if (folder) {
      p = dataAgent.updateFolder(folder.uuid, name);
    }

    p.then((res) => {
      if (res > 0) {
        busChannel.emit("getChannels");

        afterConfirm();
        handleCancel();
      }
    }).finally(() => {
      setConfirming(false);
    });
  };

  useEffect(() => {
    if (dialogStatus && inputRef && inputRef.current) {
      inputRef.current.focus();
    }

    if (action === "edit" && folder) {
      setName(folder.title);
    }
  }, [dialogStatus]);

  return (
    <Dialog open={dialogStatus} onOpenChange={setDialogStatus}>
      {trigger && <DialogTrigger asChild>{trigger}</DialogTrigger>}
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle className="text-2xl flex items-center">
            {action === "add" ? "Create Folder" : "Edit Folder"}
          </DialogTitle>
          <DialogDescription>
            {action === "add"
              ? "Organize your subscribes"
              : "update your folder"}
          </DialogDescription>
        </DialogHeader>
        <div className="pb-5">
          <div className="mb-3">
            <Input
              type="text"
              value={name}
              onChange={(e: ChangeEvent<HTMLInputElement>) =>
                handleNameChange(e.target.value)
              }
              ref={inputRef}
            />
          </div>
          <div>
            <Button
              className="w-full"
              onClick={handleSave}
              disabled={confirming}
            >
              {confirming ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Saving
                </>
              ) : (
                "Save"
              )}
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
};
