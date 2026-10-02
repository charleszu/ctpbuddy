// CTPBuddy market-data shim exports. See exports_td.cpp for the export story.
#include "ThostFtdcMdApi.h"

#include "api_md.hpp"

CThostFtdcMdApi* CThostFtdcMdApi::CreateFtdcMdApi(const char* pszFlowPath, const bool bIsUsingUdp,
                                                   const bool bIsMulticast, bool bIsProductionMode) {
    (void)pszFlowPath;
    (void)bIsUsingUdp;      // TCP only: the core speaks the CB wire protocol
    (void)bIsMulticast;
    (void)bIsProductionMode;
    return new ctpbuddy::MdApi();
}

const char* CThostFtdcMdApi::GetApiVersion() { return "CTPBuddy/0.1.0"; }
