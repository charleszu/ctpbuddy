// CTPBuddy trader shim exports. Built with LIB_TRADER_API_EXPORT so the SDK
// header marks the class dllexport: the factory symbols come out mangled
// exactly like the vendor DLL's (same-name drop-in replacement).
#include "ThostFtdcTraderApi.h"

#include "api_td.hpp"

CThostFtdcTraderApi* CThostFtdcTraderApi::CreateFtdcTraderApi(const char* pszFlowPath,
                                                               bool bIsProductionMode) {
    (void)pszFlowPath;      // no local flow files: the core owns the journal
    (void)bIsProductionMode;
    return new ctpbuddy::TraderApi();
}

const char* CThostFtdcTraderApi::GetApiVersion() { return "CTPBuddy/0.1.0"; }
