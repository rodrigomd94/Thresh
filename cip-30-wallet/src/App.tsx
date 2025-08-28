import { WelcomeScreen } from "./components/WelcomeScreen";
import { TransactionPasswordListener } from "./components/PasswordDialog";
import "./App.css";

function App() {
  return (
    <>
      <WelcomeScreen />
      <TransactionPasswordListener />
    </>
  );
}

export default App;
