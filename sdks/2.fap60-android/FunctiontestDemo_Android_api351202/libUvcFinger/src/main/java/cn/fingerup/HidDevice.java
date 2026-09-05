package cn.fingerup;

import android.app.PendingIntent;
import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;
import android.content.IntentFilter;
import android.hardware.usb.UsbDevice;
import android.hardware.usb.UsbDeviceConnection;
import android.hardware.usb.UsbEndpoint;
import android.hardware.usb.UsbInterface;
import android.hardware.usb.UsbManager;
import android.os.Build;
import android.util.Log;
import android.widget.Toast;
import java.util.HashMap;
import java.util.Iterator;


public class HidDevice
{
    private static final String ACTION_USB_PERMISSION = "com.scanner.USB_PERMISSION";//定义常量

    private static final int    PID = 0XE010;
    private static final int    VID = 0X1A86;
    public static final int ERR_OK          =       0;
    public static final int ERR_SUCCESS	    =		1;					/* 成功操作 */
    public static final int ERR_PARA		=		-1;					/* 参数错误 */
    public static final int ERR_NMEM		=		-2;					/* 内存不足 */
    public static final int ERR_NSUP		=		-3;					/* 不支持令 */
    public static final int ERR_NLNK		=		-4;					/* 设备断开 */
    public static final int ERR_NINI		=		-5;					/* 未初始化 */
    public static final int ERR_ECOD		=		-6;					/* 错误号错 */
    public static final int ERR_SERVICE	    =		-6;					/* 获取USB服务失败 */
    public static final int ERR_ELEAVE      =       -8;
    public static final int ERR_OTHR		=		-9;					/* 其它错误 */
    public static final int ERR_TIMEOUT     =	    -10;
    public static final int ERR_PERMISSION  =	    -11;
    public static final int ERR_DIFF        =       -12;				//手指相似度过低
    public static final int ERR_REFUSE		=       -13;				//用户拒绝给予权限
    private static final int MAX_WRITE_LEN       = 65;
    private static final int MAX_READ_LEN        = 65;
    private static final int READ_TIMEOUT        = 50;


    private static Context mContext;

    private UsbManager              manager;   //USB管理器
    private UsbDevice               mUsbDevice;  //找到的USB设备

    private UsbInterface            mInterface;
    private UsbDeviceConnection     mDeviceConnection;
    private UsbEndpoint             epOut;
    private UsbEndpoint             epIn;
    //private IPermissionCallback mCallbask;
    private boolean mIsDeviceOpened = false;
    static HidDevice instance;//类的单例


    private HidDevice(){};//构造函数前的修饰符为私有；这样就不能在类的外部用new来生成//一个新的对象

    public static HidDevice getInstance(Context c)//返回 单例
    {
        if(instance == null)
        {
            instance = new HidDevice();

            mContext = c;
        }

        return instance;
    }

    private final BroadcastReceiver mUsbPermissionActionReceiver = new BroadcastReceiver()
    {
        public void onReceive(Context context, Intent intent)
        {
            String action = intent.getAction();
            if (ACTION_USB_PERMISSION.equals(action)) {
                synchronized (context) {
                    boolean hasPermission = manager.hasPermission(mUsbDevice);
                    if(hasPermission){
                        isusbfinshed = 100;
                        return;
                    }
                    else {
                        mUsbDevice = null;
                        isusbfinshed = 2;
                        return ;
                    }
                }
            }
            //Toast.makeText(mContext, String.valueOf("777777777777777777777777777" ), Toast.LENGTH_LONG).show();
//            String action = intent.getAction();
//            if (ACTION_USB_PERMISSION.equals(action))
//            {
//                synchronized (this)
//                {
//                    UsbDevice usbDevice = (UsbDevice)intent.getParcelableExtra(UsbManager.EXTRA_DEVICE);
//                    if (intent.getBooleanExtra(UsbManager.EXTRA_PERMISSION_GRANTED, false))
//                    {
//                        //user choose YES for your previously popup window asking for grant perssion for this usb device
//                        if(null != usbDevice)
//                        {
//                            int nRet = OpenDevice();
//
//                            if(ERR_SUCCESS != nRet)
//                            {
//                                mCallbask.execute(nRet);
//                            }
//
//                            mIsDeviceOpened = true;
//
//                            mCallbask.execute(ERR_SUCCESS);
//                        }
//                    }
//                    else
//                    {
//                        mCallbask.execute(ERR_REFUSE);
//                    }
//
//                    mContext.unregisterReceiver(mUsbPermissionActionReceiver);
//                }
//            }
        }
    };
    public int isusbfinshed = 0;
    private int  tryGetUsbPermission()
    {
        isusbfinshed = 1;
        PendingIntent mPermissionIntent;

        int flag = PendingIntent.FLAG_UPDATE_CURRENT;

        if (android.os.Build.VERSION.SDK_INT >= 31)
        {
            flag |= (PendingIntent.FLAG_IMMUTABLE );
        }
        mPermissionIntent = PendingIntent.getBroadcast(mContext, 0, new Intent(ACTION_USB_PERMISSION), flag);
        IntentFilter filter = new IntentFilter(ACTION_USB_PERMISSION);
        if (Build.VERSION.SDK_INT >= 31){
            mContext.registerReceiver(mUsbPermissionActionReceiver, filter,Context.RECEIVER_EXPORTED);
        }
        else {
            mContext.registerReceiver(mUsbPermissionActionReceiver, filter);
        }


        //this line will let android popup window, ask user whether to allow this app to have permission to operate this usb device
        manager.requestPermission(mUsbDevice, mPermissionIntent);
        while (isusbfinshed==1) {
            Sleep(10);
            if(isusbfinshed == 2)break;
            if(isusbfinshed == 3)break;
        }
        if(isusbfinshed == 100)
            return 1;
        else
            return 0;

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
    //用UsbDeviceConnection 与 UsbInterface 进行端点设置和通讯
    private boolean getEndpoint(UsbDeviceConnection connection, UsbInterface intf)
    {
        boolean findEndpoint = false;

        if (intf.getEndpoint(1) != null)
        {
            //addText("找到输出Endpoint");
            epOut = intf.getEndpoint(1);

            findEndpoint = true;
        }

        if(!findEndpoint)
        {
            return findEndpoint;
        }

        findEndpoint = false;

        if (intf.getEndpoint(0) != null)
        {
            findEndpoint = true;
            epIn = intf.getEndpoint(0);
        }

        if(!findEndpoint)
        {
            return findEndpoint;
        }

        //Toast.makeText(mContext, String.valueOf("打开设备成功" ), Toast.LENGTH_LONG).show();

        return findEndpoint;
    }

    private int FindDevice()
    {
        // 获取USB设备
        manager = (UsbManager) mContext.getSystemService(Context.USB_SERVICE);

        if (manager == null)
        {
            Toast.makeText(mContext, String.valueOf("Permission denied for device" ), Toast.LENGTH_LONG).show();
            return ERR_SERVICE;
        }

        HashMap<String, UsbDevice> deviceList = manager.getDeviceList();

        //Toast.makeText(mContext, String.valueOf("deviceList : " + deviceList.size() ), Toast.LENGTH_LONG).show();

        Iterator<UsbDevice> deviceIterator = deviceList.values().iterator();

        while (deviceIterator.hasNext())
        {
            UsbDevice device = deviceIterator.next();

            // 在这里添加处理设备的代码
            if (device.getVendorId() == VID && device.getProductId() == PID)
            {
                //IntentFilter intentFilter;

                //(intentFilter = new IntentFilter(this.h)).addAction("android.hardware.usb.action.USB_DEVICE_DETACHED");

                //this.mContext.registerReceiver(this.t, intentFilter);


                //Toast.makeText(mContext, String.valueOf("VID : " + device.getVendorId() + " PID : " + device.getProductId() ), Toast.LENGTH_LONG).show();
                mUsbDevice = device;
                break;
            }
        }

        if (mUsbDevice == null)
        {
            return ERR_NLNK;
        }

        for (int i = 0; i < mUsbDevice.getInterfaceCount();)
        {
            // 获取设备接口，一般都是一个接口，你可以打印getInterfaceCount()方法查看接
            // 口的个数，在这个接口上有两个端点，OUT 和 IN
            UsbInterface intf = mUsbDevice.getInterface(i);

            mInterface = intf;
            break;
        }

        if(mInterface == null)
        {
            return ERR_NLNK;
        }

        return ERR_SUCCESS;
    }

    public int OpenDevice()
    {
        UsbDeviceConnection connection = null;

        // 打开设备，获取 UsbDeviceConnection 对象，连接设备，用于后面的通讯
        connection = manager.openDevice(mUsbDevice);

        if (connection == null)
        {
            return ERR_NLNK;
        }

        if (!connection.claimInterface(mInterface, true))
        {
            connection.close();

            return ERR_NLNK;
        }

        mDeviceConnection = connection;
        //用UsbDeviceConnection 与 UsbInterface 进行端点设置和通讯
        if(!getEndpoint(mDeviceConnection,mInterface))
        {
            connection.close();

            return ERR_NLNK;
        }

        return ERR_SUCCESS;
    }


    public int Init_Device()
    {
        if(mIsDeviceOpened)
        {
            return ERR_SUCCESS;
        }

        int nRet = FindDevice();

        if(ERR_SUCCESS != nRet)
        {
            return nRet;
        }

        if(!manager.hasPermission(mUsbDevice))
        {
            //mCallbask = callback;

            if(tryGetUsbPermission() == 0)
                return ERR_PERMISSION;
        }

        nRet = OpenDevice();

        if(ERR_SUCCESS != nRet)
        {
            return nRet;
        }

        mIsDeviceOpened = true;

        return ERR_SUCCESS;
    }

    public void Close_Device()
    {
        if(!mIsDeviceOpened)
        {
            return ;
        }

        if(null != mDeviceConnection )
        {
            if(true!= mDeviceConnection.releaseInterface(mInterface))
            {
                ;// throw  new IllegalArgumentException("connection releaseInterface error");
            }

            try {
                mIsDeviceOpened = false;

                mDeviceConnection.close();
                mDeviceConnection = null;
            }catch (Exception e){

            }

            return;
        }
    }

    int hid_read_timeout(byte [] data , int len , int timeout)
    {
        return mDeviceConnection.bulkTransfer(epIn, data, len, timeout);
    }

    private final static byte[] hex = "0123456789ABCDEF".getBytes();

    // 从字节数组到十六进制字符串转换
    public static String Bytes2HexString(byte[] b , int len)
    {
        byte[] buff = new byte[3 * len];

        for (int i = 0; i < len; i++)
        {
            buff[3 * i] = hex[(b[i] >> 4) & 0x0f];
            buff[3 * i + 1] = hex[b[i] & 0x0f];
            buff[3 * i + 2] = ' ';
        }

        return new String(buff);
    }


    //检查接收数据包有效性
    private boolean DoPacketCheck(byte [] packet , byte cmd)
    {
        //检测包头
        if(packet[0] != (byte)0XAA || packet[1] != (byte)0XAA || packet[2] != (byte)0X55 || packet[3] != (byte)0X55)
        {
            return false;
        }

        if(cmd != packet[4])
        {
            return false;
        }

        if(1 != packet[5])
        {
            //return false;
        }

        //检查校验和
        byte checkValue = (byte) (packet[4] + packet[6] + packet[7]);

        if(checkValue != packet[8])
        {
            //return false;
        }

        return true;
    }

    short Rx_CRC_CCITT(byte [] puchMsg, int usDataLen) // 生成 CRC
    {
        short wCRCin = 0x0000;
        short wCPoly = 0x1021;
        byte  wChar = 0;

        for(int i = 0 ; i < usDataLen ; ++i)
        {
            wChar = puchMsg[i];

            wCRCin ^= (wChar << 8);

            for(int j = 0; j < 8; j++)
            {
                if ((wCRCin & 0x8000) != 0)
                {
                    wCRCin = (short) ((wCRCin << 1) ^ wCPoly);
                }
                else
                {
                    wCRCin = (short) (wCRCin << 1);
                }
            }
        }
        return (wCRCin);
    }

    int BuildPacket(byte [] Sendbuff , byte [] data , int len)
    {
        short Send_CRC = Rx_CRC_CCITT(data,len);

        Sendbuff[len + 2] = (byte) (Send_CRC>>8 & 0xff);
        Sendbuff[len + 3] = (byte) (Send_CRC & 0xff);

        Sendbuff[0]		= 0X09;
        Sendbuff[1]		= (byte) 0xAA;

        System.arraycopy(data , 0 , Sendbuff , 2,len);

        Sendbuff[len+4] = (byte) 0xE4;
        Sendbuff[len+5] = 0x1B;
        Sendbuff[len+6] = 0x11;
        Sendbuff[len+7] = (byte) 0xEE;

        return len + 8;
    }

    public int LcdImage(int index)
    {
        if(!mIsDeviceOpened)
        {
            return ERR_NINI;
        }

        byte [] cmdData = new byte[2];

        cmdData[0] = (byte) 0X80;
        cmdData[1] = (byte) index;

        byte [] sendBuf = new byte[10];

        BuildPacket(sendBuf , cmdData , 2);

        Log.i("111" , Bytes2HexString(sendBuf , 10));


        byte [] recvBuf = new byte[64];

        boolean isReadSuccess = false;

        for(int i = 0 ; i < 5 ; ++i)
        {
            int result = mDeviceConnection.bulkTransfer(epOut, sendBuf, 10, READ_TIMEOUT);

            if(result <= 0)
            {
                //Log.i("bulkTransfer","result <= 0");
                continue;
            }

            result = mDeviceConnection.bulkTransfer(epIn, recvBuf, 64, READ_TIMEOUT);

            if(result <= 0)
            {
                //Log.i("bulkTransfer epIn","result <= 0");
                //((MainActivity)mContext).logMessageThread("bulkTransfer read : " + result);
                continue;
            }

            Log.i("bulkTransfer",Bytes2HexString(recvBuf , 64));

            if(!DoPacketCheck(recvBuf , (byte) 0X0B))
            {
                //continue;
            }

            isReadSuccess = true;

            break;
        }

        if(!isReadSuccess)
        {
            return ERR_OTHR;
        }

        return 2;

    }


    public int LedLight(int index)
    {
        if(!mIsDeviceOpened)
        {
            return ERR_NINI;
        }

        //设备现在有些问题,所以每次设置之前要关闭上次的所有灯
        byte [] clearCmdData = new byte[32];

        clearCmdData[0] = 0X02;
        clearCmdData[1] = 0X08;
        clearCmdData[2] = 0X00;

        boolean isReadSuccess = false;

        for(int i = 0 ; i < 5 ; ++i)
        {
            int result = mDeviceConnection.bulkTransfer(epOut, clearCmdData, 10, READ_TIMEOUT);

            if(result <= 0)
            {
                continue;
            }

            isReadSuccess = true;

            break;
        }

        try
        {
            Thread.sleep(5);
        }
        catch (InterruptedException e)
        {
            e.printStackTrace();
        }


        clearCmdData[1] = (byte) 0X88;

        isReadSuccess = false;

        for(int i = 0 ; i < 5 ; ++i)
        {
            int result = mDeviceConnection.bulkTransfer(epOut, clearCmdData, 10, READ_TIMEOUT);

            if(result <= 0)
            {
                continue;
            }

            isReadSuccess = true;

            break;
        }

        try
        {
            Thread.sleep(5);
        }
        catch (InterruptedException e)
        {
            e.printStackTrace();
        }

        //0作为关灯指令
        if(0 == index)
        {
            if(isReadSuccess)
            {
                return 1;
            }

            return -1;
        }


        byte [] cmdData = new byte[32];

        cmdData[0] = 0X02;

        switch(index)
        {
            case FUFingerSDK.LEFT_FOUR_FINGER:	//左手四指
                cmdData[1] = (byte) 0X88;
                cmdData[2] = 0X0F;
                break;

            case FUFingerSDK.RIGHT_FOUR_FINGER: //右手四指
                cmdData[1] = 0X08;
                cmdData[2] = 0X0F;
                break;

            case FUFingerSDK.TWO_THUMB_FINGER:  //双手拇指
                cmdData[1] = 0X08;
                cmdData[2] = 0X30;
                break;

            case FUFingerSDK.LEFT_THUMB_FINGER:  //左手拇指
                cmdData[1] = 0X08;
                cmdData[2] = 0X10;
                break;

            case FUFingerSDK.LEFT_INDEX_FINGER:  //左手食指
                cmdData[1] = (byte) 0X88;
                cmdData[2] = 0X01;
                break;

            case FUFingerSDK.LEFT_MIDDLE_FINGER: //左手中指
                cmdData[1] = (byte) 0X88;
                cmdData[2] = 0X02;
                break;

            case FUFingerSDK.LEFT_RING_FINGER:  //左手环指
                cmdData[1] = (byte) 0X88;
                cmdData[2] = 0X04;
                break;

            case FUFingerSDK.LEFT_LITTLE_FINGER: //左手小指
                cmdData[1] = (byte) 0X88;
                cmdData[2] = 0X08;
                break;

            case FUFingerSDK.RIGHT_THUMB_FINGER:  //右手拇指
                cmdData[1] = 0X08;
                cmdData[2] = 0X20;
                break;

            case FUFingerSDK.RIGHT_INDEX_FINGER:  //右手食指
                cmdData[1] = 0X08;
                cmdData[2] = 0X08;
                break;

            case FUFingerSDK.RIGHT_MIDDLE_FINGER:  //右手中指
                cmdData[1] = 0X08;
                cmdData[2] = 0X04;
                break;

            case FUFingerSDK.RIGHT_RING_FINGER:    //右手环指
                cmdData[1] = 0X08;
                cmdData[2] = 0X02;
                break;

            case FUFingerSDK.RIGHT_LITTLE_FINGER:  //右手小指
                cmdData[1] = 0X08;
                cmdData[2] = 0X01;
                break;

            default:
                break;
        }

        Log.i("cmdData",Bytes2HexString(cmdData , 32));

        isReadSuccess = false;

        for(int i = 0 ; i < 5 ; ++i)
        {
            int result = mDeviceConnection.bulkTransfer(epOut, cmdData, 10, READ_TIMEOUT);

            if(result <= 0)
            {
                continue;
            }

            isReadSuccess = true;

            break;
        }

        if(!isReadSuccess)
        {
            return -1;
        }

        return 1;
    }

    public int SetLcdScreenModeBaudRate()
    {
        if(!mIsDeviceOpened)
        {
            return ERR_NINI;
        }

        boolean isReadSuccess = false;

        for(int i = 0 ; i < 5 ; ++i)
        {
            byte[] arrayOfByte1 = new byte[32];

            byte[] arrayOfByte2 = new byte[5];

            arrayOfByte2[2] = -125;
            arrayOfByte2[3] = -52;
            arrayOfByte2[4] = 16;
            arrayOfByte2[0] = (byte)(arrayOfByte2[0] | 0x1);

            arrayOfByte2[1] = (byte)(arrayOfByte2[1] | 0x3);

            arrayOfByte2[1] = (byte)(arrayOfByte2[1] | 0x4);

            arrayOfByte2[1] = (byte)(arrayOfByte2[1] & 0xFFFFFFF7);

            arrayOfByte2[0] = -1;
            arrayOfByte2[1] = (byte)(arrayOfByte2[1] | 0xC0);
            arrayOfByte1[0] = arrayOfByte2[0];
            arrayOfByte1[1] = arrayOfByte2[1];
            arrayOfByte1[2] = arrayOfByte2[2];
            arrayOfByte1[3] = arrayOfByte2[3];
            arrayOfByte1[4] = 0X30;//arrayOfByte2[4];


            Log.i("bulkTransferLCDDDDDDDDDD",Bytes2HexString(arrayOfByte1 , 32));

            int result = mDeviceConnection.controlTransfer(32, 9, 512, 0, arrayOfByte1, 32, READ_TIMEOUT);

            if(result <= 0)
            {
                continue;
            }

            Log.i("bulkTransfer" , "result : " + result);

            isReadSuccess = true;

            break;
        }

        if(!isReadSuccess)
        {
            return ERR_OTHR;
        }

        return 1;
    }


    //LED指示灯使用的串口配置与液晶屏配置不同
    public int SetLedModeBaudRate()
    {
        if(!mIsDeviceOpened)
        {
            return ERR_NINI;
        }

        boolean isReadSuccess = false;

        for(int i = 0 ; i < 5 ; ++i)
        {
            byte[] baudRateBuf = new byte[32];

            baudRateBuf[0] = (byte) 0XFF;
            baudRateBuf[1] = (byte) 0XCF;
            baudRateBuf[2] = (byte) 0X82;
            baudRateBuf[3] = (byte) 0XD9;
            baudRateBuf[4] = (byte) 0X30;


            Log.i("bulkTransfer",Bytes2HexString(baudRateBuf , 32));

            int result = mDeviceConnection.controlTransfer(32, 9, 512, 0, baudRateBuf, 32, READ_TIMEOUT);

            if(result <= 0)
            {
                continue;
            }

            Log.i("bulkTransfer" , "result : " + result);

            isReadSuccess = true;

            break;
        }

        if(!isReadSuccess)
        {
            return ERR_OTHR;
        }

        //成功后进行初始化
        byte[] cmdData = new byte[32];

        cmdData[0] = 0X02;
        cmdData[1] = (byte) 0X88;
        cmdData[2] = (byte) 0X00;

        isReadSuccess = false;

        Log.i("cmdData",Bytes2HexString(cmdData , 32));

        for(int i = 0 ; i < 5 ; ++i)
        {

            int result = mDeviceConnection.bulkTransfer(epOut, cmdData, 32, READ_TIMEOUT);

            if(result <= 0)
            {
                continue;
            }

            isReadSuccess = true;

            break;
        }

        if(!isReadSuccess)
        {
            return -1;
        }

        try
        {
            Thread.sleep(5);
        }
        catch (InterruptedException e)
        {
            e.printStackTrace();
        }

        cmdData[1] = (byte) 0X08;
        cmdData[2] = (byte) 0X00;

        Log.i("cmdData",Bytes2HexString(cmdData , 32));

        isReadSuccess = false;

        for(int i = 0 ; i < 5 ; ++i)
        {
            int result = mDeviceConnection.bulkTransfer(epOut, cmdData, 32, READ_TIMEOUT);

            if(result <= 0)
            {
                continue;
            }

            isReadSuccess = true;

            break;
        }

        if(!isReadSuccess)
        {
            return -1;
        }

        try
        {
            Thread.sleep(5);
        }
        catch (InterruptedException e)
        {
            e.printStackTrace();
        }

        cmdData[1] = (byte) 0X18;
        cmdData[2] = (byte) 0XFF;

        Log.i("cmdData",Bytes2HexString(cmdData , 32));

        isReadSuccess = false;

        for(int i = 0 ; i < 5 ; ++i)
        {
            int result = mDeviceConnection.bulkTransfer(epOut, cmdData, 32, READ_TIMEOUT);

            if(result <= 0)
            {
                continue;
            }

            isReadSuccess = true;

            break;
        }

        if(!isReadSuccess)
        {
            return -1;
        }

        return 1;
    }
}
