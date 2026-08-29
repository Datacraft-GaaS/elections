package com.ZHIANG;

import java.util.HashMap;
import android.app.PendingIntent;
import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;
import android.content.IntentFilter;
import android.hardware.usb.UsbConstants;
import android.hardware.usb.UsbDevice;
import android.hardware.usb.UsbDeviceConnection;
import android.hardware.usb.UsbEndpoint;
import android.hardware.usb.UsbInterface;
import android.hardware.usb.UsbManager;
import android.os.Build;
import android.util.Log;

public class HostUsb {
    private static final String TAG = "OpenHostUsb";
    private static final boolean D = false;
	private static final String ACTION_USB_PERMISSION = "com.ZHIANG.USB_PERMISSION";
    private Context context = null;
    private UsbManager mDevManager = null;
    private UsbInterface intf = null;
    private UsbDeviceConnection connection = null;
    private UsbDevice devicett = null;



    private int m_nEPOutSize = 2048;
    private int m_nEPInSize = 2048;
    private byte[] m_abyTransferBuf = new byte[2048];

    UsbEndpoint endpoint_IN = null;
    UsbEndpoint endpoint_OUT = null;
    UsbEndpoint endpoint_INT = null;
    UsbEndpoint curEndpoint = null;
    private int isusbfinshed = 0;

    private  static void writeLog(String fileName,String content) {
        Log.e(TAG,content);
    }
    public HostUsb() {    }
    public boolean AuthorizeDevice(Context paramContext, int VID, int PID) {
        isusbfinshed = 1;
        context = paramContext;
        mDevManager = ((UsbManager) context.getSystemService(Context.USB_SERVICE));
        HashMap<String, UsbDevice> deviceList = mDevManager.getDeviceList();
        if (D) writeLog(TAG, "AuthorizeDevice --------------------- news:" + "mDevManager");
        for (UsbDevice tdevice : deviceList.values()) {
            if (D) writeLog(TAG, "AuthorizeDevice --------------------- news:" + tdevice);
            if (D)
                writeLog(TAG, "news:" + "usb VID:" + tdevice.getVendorId() + "  PID:" + tdevice.getProductId());
            if (tdevice.getVendorId() == VID && (tdevice.getProductId() == PID)) {
                boolean hasPermission = mDevManager.hasPermission(tdevice);
                if (!hasPermission) {
                    Intent intent = new Intent(ACTION_USB_PERMISSION);
                    int flags =  PendingIntent.FLAG_UPDATE_CURRENT;
                    if (Build.VERSION.SDK_INT >= 31) {
                        flags |= (PendingIntent.FLAG_IMMUTABLE );
                    }
                    PendingIntent permissionIntent = PendingIntent.getBroadcast(context, 0, intent,flags);
                    IntentFilter filter = new IntentFilter(ACTION_USB_PERMISSION);
                    //context.registerReceiver(mUsbReceiver, filter);
                    if (Build.VERSION.SDK_INT >= 31) {
                        context.registerReceiver(mUsbReceiver, filter, Context.RECEIVER_EXPORTED);
                    }
                    else {
                        context.registerReceiver(mUsbReceiver, filter);
                    }
                    mDevManager.requestPermission(tdevice, permissionIntent);

                    devicett  = tdevice;
                    Log.e("aaa","devicett = "+devicett);
                    while (isusbfinshed==1) {
                        Sleep(10);
                        if(isusbfinshed == 2)break;
                        if(isusbfinshed == 3)break;
                    }
                    if(isusbfinshed == 100)
                        return true;
                    else
                        return false;


                } else {
                    isusbfinshed = 100;
                    devicett = tdevice;
                    if (D) writeLog(TAG, "permission denied for device " + devicett);
                    return true;
                }
            }
        }
        return false;
    }
    private void Sleep(int times)
    {
        try {
            Thread.sleep(times);
        } catch (InterruptedException e) {
            // TODO Auto-generated catch block
            e.printStackTrace();
        }
    }
    private final BroadcastReceiver mUsbReceiver = new BroadcastReceiver() {
        public void onReceive(Context context, Intent intent) {
            String action = intent.getAction();
            if (ACTION_USB_PERMISSION.equals(action)) {
                synchronized (context) {
                    boolean hasPermission = false;
                    if(devicett!=null) {
                        hasPermission = mDevManager.hasPermission(devicett);
                    }
                    if(hasPermission){
                        isusbfinshed = 100;
                        return;
                    }
                    else {
                        devicett = null;
                        isusbfinshed = 2;
                        return ;
                    }
                }
            }
        }
    };

    public int OpenDeviceInterfaces() {
        if(isusbfinshed != 100)
            return -1;
        UsbDevice mDevice = devicett;
        // Log.d(TAG, "setDevice " + mDevice);
        int fd = -1;
        if (mDevice == null)
            return -1;
        connection = mDevManager.openDevice(mDevice);
        if (!connection.claimInterface(mDevice.getInterface(0), true))
            return -1;
        if (mDevice.getInterfaceCount() < 1)
            return -1;
        intf = mDevice.getInterface(0);
        if (intf.getEndpointCount() == 0)
            return -1;
        for (int i = 0; i < intf.getEndpointCount(); i++) {
            if (intf.getEndpoint(i).getType() == UsbConstants.USB_ENDPOINT_XFER_BULK) {
                if (intf.getEndpoint(i).getDirection() == UsbConstants.USB_DIR_IN) {
                    endpoint_IN = intf.getEndpoint(i);
                } else if (intf.getEndpoint(i).getDirection() == UsbConstants.USB_DIR_OUT) {
                    endpoint_OUT = intf.getEndpoint(i);
                }
            } else if (intf.getEndpoint(i).getType() == UsbConstants.USB_ENDPOINT_XFER_INT) {
                endpoint_INT = intf.getEndpoint(i);
            } else {
                if (D) writeLog(TAG, "Not Endpoint or other Endpoint ");
            }
        }
        curEndpoint = intf.getEndpoint(0);
        if ((connection != null)) {
            if (D) writeLog(TAG, "open connection success!");
            fd = connection.getFileDescriptor();
            if (D) writeLog(TAG, "fd = " + fd);
            return fd;
        } else {
            if (D) writeLog(TAG, "finger device open connection FAIL");
            return -1;
        }
    }

    public void CloseDeviceInterface() {
        if (connection != null) {
            connection.releaseInterface(intf);
            connection.close();
        }
    }

    public boolean USBBulkSend(byte[] pBuf, int nLen, int nTimeOut) {
        int i, n, r, w_nRet;
        n = nLen / m_nEPOutSize;
        r = nLen % m_nEPOutSize;
        for (i = 0; i < n; i++) {
            System.arraycopy(pBuf, i * m_nEPOutSize, m_abyTransferBuf, 0, m_nEPOutSize);
            w_nRet = connection.bulkTransfer(endpoint_OUT, m_abyTransferBuf, m_nEPOutSize, nTimeOut);
            if (w_nRet != m_nEPOutSize) {
                if (D)
                    writeLog(TAG, "USBBulkSend -------------------------------------\n bulkTransfer failure ");
                return false;
            }
        }
        if (r > 0) {
            System.arraycopy(pBuf, i * m_nEPOutSize, m_abyTransferBuf, 0, r);
            w_nRet = connection.bulkTransfer(endpoint_OUT, m_abyTransferBuf, r, nTimeOut);
            if (w_nRet != r) {
                if (D)
                    writeLog(TAG, "USBBulkSend -------------------------------------\n bulkTransfer failure ");
                return false;
            }
        }

        return true;
    }
    public boolean USBBulkReceive(byte[] pBuf, int nLen, int nTimeOut) {
        int i, n, r, w_nRet;
        n = nLen / m_nEPInSize;
        r = nLen % m_nEPInSize;
        for (i = 0; i < n; i++) {
            w_nRet = connection.bulkTransfer(endpoint_IN, m_abyTransferBuf, m_nEPInSize, nTimeOut);
            if (w_nRet != m_nEPInSize) {
                if (D)
                    writeLog(TAG, "USBBulkReceive -------------------------------------\n bulkTransfer failure ");
                return false;// w_nRet = connection.bulkTransfer(endpoint_IN,
                // m_abyTransferBuf, m_nEPInSize,
                // nTimeOut);//i--;//
            }
            System.arraycopy(m_abyTransferBuf, 0, pBuf, i * m_nEPInSize, m_nEPInSize);
        }
        if (r > 0) {
            w_nRet = connection.bulkTransfer(endpoint_IN, m_abyTransferBuf, r, nTimeOut);
            if (w_nRet != r) {
                if (D)
                    writeLog(TAG, "USBBulkReceive -------------------------------------\n bulkTransfer failure ");
                // w_nRet = connection.bulkTransfer(endpoint_IN,
                // m_abyTransferBuf, r, nTimeOut);
                return false;
            }
            System.arraycopy(m_abyTransferBuf, 0, pBuf, i * m_nEPInSize, r);
        }
        return true;
    }


}
