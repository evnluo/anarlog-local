import type { ReactNode } from "react";

import { type BillingAccess, BillingContext } from "./billing-context";

const LOCAL_BILLING_ACCESS: BillingAccess = Object.freeze({
  entitlements: [],
  subscriptionStatus: null,
  isPro: false,
  isLite: false,
  isPaid: false,
  isTrialing: false,
  isPaused: false,
  hasPaymentMethod: false,
  trialEnd: null,
  trialDaysRemaining: null,
  cancelAtPeriodEnd: false,
  currentPeriodEnd: null,
  plan: "free",
  isReady: true,
  canStartTrial: { data: false, isPending: false },
  upgradeToPro: () => {},
  isUpgradingToPro: false,
});

export function LocalBillingProvider({ children }: { children: ReactNode }) {
  return (
    <BillingContext.Provider value={LOCAL_BILLING_ACCESS}>
      {children}
    </BillingContext.Provider>
  );
}
