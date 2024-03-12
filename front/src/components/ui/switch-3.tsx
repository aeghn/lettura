import React from "react";
import { Icon } from "../Icon";
import {
  LucideBell,
  LucidePackage,
  LucideSalad,
  LucideSparkles,
  LucideStar,
  LucideWallet,
  LucideWallet2,
  RefreshCw,
} from "lucide-react";

export interface TSBProps {
  name1: String;
  name2: String;
  name3: String;
  onStageOne: () => void;
  onStageTwo: () => void;
  onStageThree: () => void;
  initValue: number;
  iconSize: number;
}

export const ThreeStageButton = ({ props }: { props: TSBProps }) => {
  const [currentStage, setCurrentStage] = React.useState(props.initValue);

  const handleStageOne = () => {
    setCurrentStage(0);
    props.onStageOne();
  };

  const handleStageTwo = () => {
    setCurrentStage(1);
    props.onStageTwo();
  };

  const handleStageThree = () => {
    setCurrentStage(2);
    props.onStageThree();
  };

  const iconSize = props.iconSize;

  const styleGen = (value: number) => {
    return `${
      currentStage === value ? "bg-blue-200 text-black" : ""
    } px-1 m-1 py-2 rounded-md flex flex-col text-xs w-full`;
  };

  return (
    <div className="flex flex-row  items-center rounded-sm justify-around m-0 p-1 ">
      <button onClick={handleStageOne} className={styleGen(0)}>
        <LucideStar size={iconSize} className="w-full mr-1" />
        {props.name1}
      </button>
      <button onClick={handleStageTwo} className={styleGen(1)}>
        <LucideSparkles size={iconSize} className="w-full mr-1" />
        {props.name2}
      </button>
      <button onClick={handleStageThree} className={styleGen(2)}>
        <LucidePackage size={iconSize} className="w-full mr-1" />
        {props.name3}
      </button>
    </div>
  );
};
