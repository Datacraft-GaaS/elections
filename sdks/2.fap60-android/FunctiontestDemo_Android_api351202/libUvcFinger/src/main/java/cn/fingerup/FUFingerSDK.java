package cn.fingerup;

import android.annotation.SuppressLint;
import android.app.Activity;
import android.app.FragmentTransaction;
import android.content.Context;
import android.graphics.SurfaceTexture;
import android.hardware.usb.UsbDevice;
import android.os.Bundle;
import android.os.Handler;
import android.util.Log;
import android.view.Gravity;
import android.view.LayoutInflater;
import android.view.Surface;
import android.view.View;
import android.view.ViewGroup;
import android.view.Window;
import android.view.WindowManager;
import android.widget.FrameLayout;
import android.widget.Toast;

import com.serenegiant.common.BaseFragment;
import com.serenegiant.usb.IFrameCallback;
import com.serenegiant.usb.USBMonitor;
import com.serenegiant.usb.UVCCamera;
import com.serenegiant.widget.SimpleUVCCameraTextureView;

import java.nio.ByteBuffer;
import java.util.LinkedList;
import java.util.List;
import java.util.Queue;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;


public class FUFingerSDK extends BaseFragment
{
    class RawData
    {
        public int w;
        public int h;
        public byte[] data;
    }
    private static boolean DEBUG = true;  // set false when production
    private static final String TAG = FUFingerSDK.class.getSimpleName();

    //public static final int PID                             = 0X1001;
    //public static final int VID                             = 0X1BCF;

    public static final int LEFT_FOUR_FINGER                 = 2;
    public static final int RIGHT_FOUR_FINGER                = 3;
    public static final int TWO_THUMB_FINGER                 = 4;
    public static final int LEFT_THUMB_FINGER                = 5;
    public static final int LEFT_INDEX_FINGER                = 6;
    public static final int LEFT_MIDDLE_FINGER               = 7;
    public static final int LEFT_RING_FINGER                 = 8;
    public static final int LEFT_LITTLE_FINGER               = 9;
    public static final int RIGHT_THUMB_FINGER               = 10;
    public static final int RIGHT_INDEX_FINGER               = 11;
    public static final int RIGHT_MIDDLE_FINGER              = 12;
    public static final int RIGHT_RING_FINGER                = 13;
    public static final int RIGHT_LITTLE_FINGER              = 14;
    public static final int LEFT_FOUR_FINGER_SUCCESS         = 15;
    public static final int LEFT_FOUR_FINGER_FAIL            = 16;
    public static final int RIGHT_FOUR_FINGER_SUCCESS        = 17;
    public static final int RIGHT_FOUR_FINGER_FAIL           = 18;
    public static final int TWO_THUMB_FINGER_SUCCESS         = 19;
    public static final int TWO_THUMB_FINGER_FAIL            = 20;
    public static final int FINGER_SUCCESS                   = 21;
    public static final int FINGER_FAIL                      = 22;



    public   int PID                           = 0X0581;
    public   int VID                           = 0X0507;

    public  int PID1                           = 0X636D;
    public  int VID1                           = 0X0C45;

    public static final int ORG_WIDTH                       = 2688;
    public static final int ORG_HEIGHT                      = 1944;

    public static final int FINGER_WIDTH                    = 1600;
    public static final int FINGER_HEIGHT                   = 1500;

    public static final int FLAT_FINGER_WIDTH               = 300;
    public static final int FLAT_FINGER_HEIGHT              = 400;

    public static final int ROLL_FINGER_WIDTH               = 800;
    public static final int ROLL_FINGER_HEIGHT              = 750;

    public static volatile boolean  mStartRoll              = false;

    //public static final int PID                 = 0X082D;
    //public static final int VID                 = 0X046D;

    //public static final int PID                 = 2848;//1409;
    //public static final int VID                 = 1409;//2848;  //设备未连接

    public static final int ERR_BUSY                    = -5;  //设备忙

    public static final int MAX_CHANNEL		                =       0;

    public static final int LS_ERROR_SUCCESS                =     (1);
    public static final int LS_ERROR_INVALID_PARAMETER      =     (-1);
    public static final int LS_ERROR_NOT_ENOUGH_MEMORY      =     (-2);
    public static final int LS_ERROR_NOT_SUPPORT_FUNCTION   =     (-3);
    public static final int LS_ERROR_DEVICE_NOT_FOUND       =     (-4);
    public static final int LS_ERROR_DEVICE_NOT_INIT        =     (-5);
    public static final int LS_ERROR_INVALIDE_CODE          =     (-6);
    public static final int LS_ERROR_NO_PRIVILEGE           =     (-7);
    public static final int LS_ERROR_PERMISSION_DENIED      =     (-8);//拒绝了权限申请
    public static final int LS_ERROR_OTHER				    =	  (-9);
    public static final int LS_ERROR_TIMEOUT				=	  (-10);




    public static final int RAW_FINGER_SIZE             = FINGER_WIDTH * FINGER_HEIGHT;
    public static final int BMP_FINGER_SIZE             = 1078 + FINGER_WIDTH * FINGER_HEIGHT;

    private byte[] mYuvBuffer = new byte[ORG_WIDTH * ORG_HEIGHT * 2];
    private byte[] distortionData = new byte[4096];

    private final Object mSync = new Object();
    // for accessing USB and USB camera
    private USBMonitor mUSBMonitor;
    private UVCCamera mUVCCamera;
    private SimpleUVCCameraTextureView mUVCCameraView;
    private Surface mPreviewSurface;
    private boolean mIsCaptureBusy = false;

    private USBMonitor.UsbControlBlock mUsbControlBlock;

    private boolean             mIsDeviceOpened = false;
    private Handler             mUIHandler;
    private OnConnectListener   mConnectListener;
    private IPreviewCallback    mPreviewCallback;
    private boolean             mIsLcdScreenMode = true;
    private  Context mContext;

    public Queue mRollFingerQueue = new LinkedList();

    public boolean              mNeedUpdate = true;

    HidDevice mHidInstance = null;




    public FUFingerSDK()
    {

    }

    @SuppressLint("ValidFragment")
    public FUFingerSDK(Context context)
    {
        this.mContext   = context;

        this.mUIHandler = new Handler(this.mContext.getMainLooper());

        FragmentTransaction ft = ((Activity)this.mContext).getFragmentManager().beginTransaction();

        ft.replace(android.R.id.content, this);

        ft.commit();

        mHidInstance = HidDevice.getInstance(context);

        for(int i=0;i<FINGER_WIDTH * FINGER_HEIGHT;i++){
            mCapturedBuffer[i]=(byte)255;
        }
    }

    public void LcdImage(int index)
    {
        mHidInstance.LcdImage(index);
    }

    public void LedLight(int index)
    {
        mHidInstance.LedLight(index);
    }

    public void setConnectListener(OnConnectListener connectListener)
    {
        this.mConnectListener = connectListener;
    }

    public void setPreviewCallback(IPreviewCallback callback)
    {
        mPreviewCallback = callback;
    }

    @Override
    public void onCreate(Bundle savedInstanceState)
    {
        super.onCreate(savedInstanceState);
    }

    @Override
    public View onCreateView(LayoutInflater inflater, ViewGroup container, Bundle savedInstanceState)
    {
        View rootView = createViewFromCode(inflater, container, savedInstanceState);
        Window window = getActivity().getWindow();
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);

        return rootView;
    }

    private View createViewFromCode(LayoutInflater inflater, ViewGroup container, Bundle savedInstance)
    {
        FrameLayout frameLayout = new FrameLayout(getActivity());
        FrameLayout.LayoutParams frameLayoutpm = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT);
        frameLayout.setLayoutParams(frameLayoutpm);
        frameLayoutpm.gravity = Gravity.CENTER;

        mUVCCameraView = new SimpleUVCCameraTextureView(getActivity());
        mUVCCameraView.setAspectRatio(2688 / (float) 1844);

        mUVCCameraView.setLayoutParams(new FrameLayout.LayoutParams(2,2));

        mUSBMonitor = new USBMonitor(getActivity().getApplicationContext(), mOnDeviceConnectListener);
        frameLayout.addView(mUVCCameraView);

        return frameLayout;
    }

    @Override
    public void onStart()
    {
        super.onStart();

        synchronized (mSync)
        {
            if (mUSBMonitor != null)
            {
              //  mUSBMonitor.register();
            }
        }
    }

    @Override
    public void onResume()
    {
        super.onResume();
    }

    @Override
    public void onStop()
    {
        super.onPause();

        synchronized (mSync)
        {
            if (mUVCCamera != null)
            {
                mUVCCamera.stopPreview();
            }

            if (mUSBMonitor != null)
            {
                mUSBMonitor.unregister();
            }
        }
        super.onStop();
    }

    @Override
    public void onDestroy()
    {
        synchronized (mSync)
        {
            if (DEBUG) Log.v(TAG, "onDestroy:");

            releaseCamera();

            if (mUSBMonitor != null)
            {
                mUSBMonitor.destroy();
                mUSBMonitor = null;
            }
        }
        LedLight(0);
        mUVCCameraView = null;
        mUsbControlBlock = null;
        super.onDestroy();
    }


    //
    public int LIVESCAN_Init(boolean isLcdScreenMode)
    {
        int ret = 0;
        Log.e("LAG","LIVESCAN_Init start");
        mIsLcdScreenMode = isLcdScreenMode;
        mUSBMonitor = new USBMonitor(mContext, mOnDeviceConnectListener);
        ret = mUSBMonitor.getrwusbdevices();
//        Log.e("LAG","LIVESCAN_Init ret = "+ret);
//        if(ret != 1)
//        {
//            return LS_ERROR_DEVICE_NOT_FOUND;
//        }
        if(mUSBMonitor.isPermission() == false)
        {   Log.e("LAG","isPermission fail " );
            return LS_ERROR_DEVICE_NOT_FOUND;
        }
        Log.e("LAG","isPermission success " );
        return LS_ERROR_SUCCESS;
    }

    public int LIVESCAN_Close()
    {
        //设备未打开的情况下不处理
        if(!mIsDeviceOpened)
        {
            return LS_ERROR_DEVICE_NOT_INIT;
        }

        releaseCamera();

        mHidInstance.Close_Device();

        mIsDeviceOpened = false;

        return LS_ERROR_SUCCESS;
    }

    public int LIVESCAN_GetChannelCount()
    {
        if (!mIsDeviceOpened)
        {
            return LS_ERROR_DEVICE_NOT_INIT;
        }

        return 1;
    }

    /*-----------------------------------------------------------------------------
    函数名   : LIVESCAN_IsSupportSetup
    功能     : 判断采集设备是否支持设置对话框
    参数     : 无
    返回值   : 若支持返回1, 否则返回0
    说明	 : LIVESCAN_Setup必须存在
    -----------------------------------------------------------------------------*/
    public int LIVESCAN_IsSupportSetup()
    {
        if (!mIsDeviceOpened)
        {
            return LS_ERROR_DEVICE_NOT_INIT;
        }

        return 0;
    }

    /*-----------------------------------------------------------------------------
    函数名   : LIVESCAN_BeginCapture
    功能     : 准备采集一帧图像
    参数     : [IN] nChannel 通道号
    返回值   : 成功返回1， <0返回错误代码
    说明	 :
    -----------------------------------------------------------------------------*/
    public int LIVESCAN_BeginCapture(int nChannel)
    {
        if (!mIsDeviceOpened)
        {
            return LS_ERROR_DEVICE_NOT_INIT;
        }

        if (nChannel < 0 || nChannel > MAX_CHANNEL)
        {
            return LS_ERROR_INVALID_PARAMETER;
        }

        return LS_ERROR_SUCCESS;
    }

    /*-----------------------------------------------------------------------------
    函数名   : LIVESCAN_Endpture
    功能     : 结束采集一帧图像
    参数     : [IN] nChannel 通道号
    返回值   : 成功返回1， <0返回错误代码
    说明	 :
    -----------------------------------------------------------------------------*/
    public int LIVESCAN_EndCapture(int nChannel)
    {
        if (!mIsDeviceOpened)
        {
            return LS_ERROR_DEVICE_NOT_INIT;
        }

        if (nChannel < 0 || nChannel > MAX_CHANNEL)
        {
            return LS_ERROR_INVALID_PARAMETER;
        }

        return LS_ERROR_SUCCESS;
    }

    /*-----------------------------------------------------------------------------
    函数名   : LIVESCAN_GetVersion
    功能     : 取得接口的版本
    参数     : 无
    返回值   : 当前版本为1.00， 返回值为100。若以后扩展接口，则需要修改此版本号。
    <0返回错误代码。
    说明	 : 不需要初始化就可以调用
    -----------------------------------------------------------------------------*/
    public int LIVESCAN_GetVersion()
    {
        return 100;
    }

    /*-----------------------------------------------------------------------------
    函数名   : LIVESCAN_Setup
    功能     : 调用采集仪的属性设置对话框
    参数     : 无
    返回值   : 成功返回1， <0返回错误代码
    说明	 : 模式对话框
    -----------------------------------------------------------------------------*/
    public int LIVESCAN_Setup()
    {
        return LS_ERROR_NOT_SUPPORT_FUNCTION;
    }

    /*-----------------------------------------------------------------------------
    函数名   : LIVESCAN_GetMaxImageSize
    功能     : 获得采集设备采集图像的宽度、高度的最大值
    参数     : [IN] nChannel 通道号
    [OUT] pnWidth 图像宽度
    [OUT] pnHeight 图像高度
    返回值   : 成功返回1， <0返回错误代码
    说明	 :
    -----------------------------------------------------------------------------*/
    public int LIVESCAN_GetMaxImageSize(int nChannel, int [] pnWidth, int [] pnHeight)
    {
        if (nChannel < 0 || nChannel > MAX_CHANNEL)
        {
            return LS_ERROR_INVALID_PARAMETER;
        }

        if (pnWidth == null || pnHeight == null)
        {
            return LS_ERROR_INVALID_PARAMETER;
        }

        pnWidth[0]	= FINGER_WIDTH;
        pnHeight[0]	= FINGER_HEIGHT;

        return LS_ERROR_SUCCESS;
    }



    /*-----------------------------------------------------------------------------
    函数名   : LIVESCAN_SetBright
    功能     : 设置采集设备当前的亮度
    参数     : [IN] nChannel 通道号
    [IN] nBright 亮度, 范围 0-255
    返回值   : 成功返回1， <0返回错误代码
    -----------------------------------------------------------------------------*/
    public int  LIVESCAN_SetBright(int nChannel, int nBright)
    {
        if (!mIsDeviceOpened)
        {
            return LS_ERROR_DEVICE_NOT_INIT;
        }

        if (nChannel < 0 || nChannel > MAX_CHANNEL)
        {
            return LS_ERROR_INVALID_PARAMETER;
        }

        if (nBright < 0 || nBright > 255)
        {
            return LS_ERROR_INVALID_PARAMETER;
        }

        return LS_ERROR_SUCCESS;
    }

    /*-----------------------------------------------------------------------------
    函数名   : LIVESCAN_SetContrast
    功能     : 设置采集设备当前的对比度
    参数     : [IN] nChannel 通道号
    [IN] nContrast 对比度, 范围 0-255
    返回值   : 成功返回1， <0返回错误代码
    -----------------------------------------------------------------------------*/
    public int  LIVESCAN_SetContrast(int nChannel, int nContrast)
    {
        if (!mIsDeviceOpened)
        {
            return LS_ERROR_DEVICE_NOT_INIT;
        }

        if (nChannel < 0 || nChannel > MAX_CHANNEL)
        {
            return LS_ERROR_INVALID_PARAMETER;
        }

        if (nContrast < 0 || nContrast > 255)
        {
            return LS_ERROR_INVALID_PARAMETER;
        }

        return LS_ERROR_SUCCESS;
    }

    /*-----------------------------------------------------------------------------
    函数名   : LIVESCAN_GetBright
    功能     : 获得采集设备当前的亮度
    参数     : [IN] nChannel 通道号
    [OUT] pnBright : 亮度, 范围 0-255
    返回值   : 成功返回1， <0返回错误代码
    -----------------------------------------------------------------------------*/
    public int LIVESCAN_GetBright(int nChannel, int [] pnBright)
    {
        if (!mIsDeviceOpened)
        {
            return LS_ERROR_DEVICE_NOT_INIT;
        }

        if (nChannel < 0 || nChannel > MAX_CHANNEL)
        {
            return LS_ERROR_INVALID_PARAMETER;
        }

        if (pnBright == null)
        {
            return LS_ERROR_INVALID_PARAMETER;
        }

        return LS_ERROR_SUCCESS;
    }


    /*-----------------------------------------------------------------------------
    函数名   : LIVESCAN_GetContrast
    功能     : 获得采集设备当前的对比度
    参数     : [IN] nChannel 通道号
    [OUT] pnContrast 对比度, 范围 0-255
    返回值   : 成功返回1， <0返回错误代码
    -----------------------------------------------------------------------------*/
    public int LIVESCAN_GetContrast(int nChannel, int [] pnContrast)
    {
        if (!mIsDeviceOpened)
        {
            return LS_ERROR_DEVICE_NOT_INIT;
        }

        if (nChannel < 0 || nChannel > MAX_CHANNEL)
        {
            return LS_ERROR_INVALID_PARAMETER;
        }

        if (pnContrast == null)
        {
            return LS_ERROR_INVALID_PARAMETER;
        }

        return LS_ERROR_SUCCESS;
    }

    /*-----------------------------------------------------------------------------
    函数名   : LIVESCAN_GetCaptWindow
    功能     : 获得当前图像的采集位置、宽度和高度
    参数     : [IN] nChannel 通道号
    [OUT] pnOriginX 图像采集原点坐标X值
    [OUT] pnOriginY 图像采集原点坐标Y值
    [OUT] pnWidth 图像宽度
    [OUT] pnHeight 图像高度
    返回值   : 成功返回1， <0返回错误代码
    说明	 : 警用必须为640*640, 民用必须为 440 <= w, h <= 640. 坐标原点通常为(0,0)
    -----------------------------------------------------------------------------*/
    public int LIVESCAN_GetCaptWindow(int nChannel, int [] pnOriginX, int [] pnOriginY, int [] pnWidth, int [] pnHeight)
    {
        if (!mIsDeviceOpened)
        {
            return LS_ERROR_DEVICE_NOT_INIT;
        }

        if (nChannel < 0 || nChannel > MAX_CHANNEL)
        {
            return LS_ERROR_INVALID_PARAMETER;
        }


        return LS_ERROR_SUCCESS;
    }

    /*-----------------------------------------------------------------------------
    函数名   : LIVESCAN_SetCaptWindow
    功能     : 设置当前图像的采集位置、宽度和高度 （可选）
    参数     : [IN] nChannel 通道号
    [IN] nOriginX 图像采集原点坐标X值
    [IN] nOriginY 图像采集原点坐标Y值
    [IN] nWidth 图像宽度
    [IN] nHeight 图像高度
    返回值   : 成功返回1， <0返回错误代码
    说明	 : 警用必须为640*640, 民用必须为 440 <= w, h <= 640.
    -----------------------------------------------------------------------------*/
    public int LIVESCAN_SetCaptWindow(int nChannel, int nOriginX , int nOriginY ,int nWidth , int nHeight)
    {
        if (!mIsDeviceOpened)
        {
            return LS_ERROR_DEVICE_NOT_INIT;
        }

        if (nChannel < 0 || nChannel > MAX_CHANNEL)
        {
            return LS_ERROR_INVALID_PARAMETER;
        }

        return LS_ERROR_SUCCESS;
    }
    /*-----------------------------------------------------------------------------
    函数原型：     	int __stdcall LIVESCAN_IsSupportCaptWindow(int nChannel);
    参数：         	int nChannel      通道号。输入参数。
    返回值：      	若采集接口只支持平面掌纹及侧面掌纹采集窗口的设置，则返回1,
    若采集接口支持侧面掌纹、平面掌纹、平面全掌纹采集窗口的设置，则返回2，
    若采集接口不支持设置采集窗口，则返回0。
    <0 返回错误代码，调用LIVESCAN_GetErrInfo函数获取错误信息。错误代码值符合3.22的要求。
    说明：        	判断是否可以调用LIVESCAN_SetCaptWindow函数。若不支持设置采集窗口，则
    默认的采集位置、大小可以通过LIVESCAN_GetCaptWindow来得到。
    -----------------------------------------------------------------------------*/
    public int LIVESCAN_IsSupportCaptWindow(int nChannel)
    {
        if (nChannel < 0 || nChannel > MAX_CHANNEL)
        {
            return LS_ERROR_INVALID_PARAMETER;
        }

        return 0;
    }

    public int LIVESCAN_GetFPRawData(int nChannel, byte [] pRawData)
    {
        if (!mIsDeviceOpened)
        {
            return LS_ERROR_DEVICE_NOT_INIT;
        }

        if (nChannel < 0 || nChannel > MAX_CHANNEL)
        {
            return LS_ERROR_INVALID_PARAMETER;
        }

        System.arraycopy(mCapturedBuffer , 0 , pRawData , 0 , FINGER_WIDTH * FINGER_HEIGHT);

        return LS_ERROR_SUCCESS ;
    }

    //采集一帧滚动手指为后续拼接提供数据
    public int LIVESCAN_GetRollFPRawData(byte [] pRawData , int nWidth , int nHeight)
    {
        if(mRollFingerQueue.isEmpty())
        {
            return -1;
        }

        Log.i("a" , "mRollFingerQueue.size : " + mRollFingerQueue.size());

        RawData data = (RawData) mRollFingerQueue.poll();

        FingerAlgo.GetRollFinger(data.data , FINGER_WIDTH , FINGER_HEIGHT , pRawData , nWidth , nHeight);

        return LS_ERROR_SUCCESS ;
    }

    public int LIVESCAN_GetFlatFPRawData(byte [] pRawData , int nWidth , int nHeight)
    {
        byte [] tempBuffer = new byte[FINGER_WIDTH * FINGER_HEIGHT];

        int nResult = LIVESCAN_GetFPRawData(0 , tempBuffer);

        if(LS_ERROR_SUCCESS != nResult)
        {
            return nResult;
        }

        FingerAlgo.GetFlatFinger(tempBuffer , FINGER_WIDTH , FINGER_HEIGHT , pRawData , nWidth , nHeight);

        //采集到1600*1500后再根据手指位置截取图像

        return LS_ERROR_SUCCESS ;
    }

    public int LIVESCAN_Beep(int beepType)
    {
        mUVCCamera.beep(beepType);

        return 1;
    }

    public int LIVESCAN_SetFingerDryWet(int level)
    {
        mUVCCamera.SetBacklightComp(level);

        return 0;
    }


    CountDownLatch mCaptureLatch = null;
    //AtomicInteger mNeedCaptureFinger = new AtomicInteger(0);
    volatile boolean mNeedCaptureFinger = false;

    //同步方式采集指纹 boolean detectFingerPress ,
    public int captureFinger(int timeout , byte [] capturedFinger , byte [] captureBmp)
    {
        if (!mIsDeviceOpened)
        {
            Log.i(TAG, "LS_ERROR_DEVICE_NOT_INIT...");
            return LS_ERROR_DEVICE_NOT_INIT;
        }

        if (mIsCaptureBusy)
        {
            Log.i(TAG, "mIsCaptureBusy...");
            return ERR_BUSY;
        }

        mNeedCaptureFinger = true;

        mCaptureLatch = new CountDownLatch(1);

        boolean waitOk = false;

        try
        {
            Log.i(TAG, "await...");
            waitOk = mCaptureLatch.await(timeout, TimeUnit.MILLISECONDS);
            Log.i(TAG, "await end");
        }
        catch (InterruptedException e)
        {
            e.printStackTrace();
        }

        if(!waitOk)
        {
            Log.i(TAG, "LS_ERROR_TIMEOUT...");
            return LS_ERROR_TIMEOUT;
        }

        mCaptureLatch = null;

        System.arraycopy(mCapturedFrame , 0 , capturedFinger , 0 , FINGER_WIDTH * FINGER_HEIGHT);

        return LS_ERROR_SUCCESS;
    }

    void postResult(int errorCode)
    {
        if (FUFingerSDK.this.mConnectListener != null)
        {
            FUFingerSDK.this.mUIHandler.post(new Runnable()
            {
                public void run()
                {
                    if (FUFingerSDK.this.mConnectListener != null && FUFingerSDK.this.mConnectListener instanceof OnConnectListener)
                    {
                        if(LS_ERROR_SUCCESS == errorCode)
                        {
                            ((OnConnectListener) FUFingerSDK.this.mConnectListener).onConnected();

                            mUVCCamera.checkSupportFlag(1);
                        }
                        else
                        {
                            ((OnConnectListener) FUFingerSDK.this.mConnectListener).onConnectFailed(errorCode);
                        }
                    }
                }
            });
        }
    }

    //打开摄像头采集设备,并设置预览和回调
    private void openCameraDevice(final USBMonitor.UsbControlBlock ctrlBlock)
    {
        if (mUVCCamera != null)
        {
            return;
        }
        int nRet = mHidInstance.Init_Device();
        if(HidDevice.ERR_SUCCESS == nRet)
        {
            //使用LCD 液晶屏控制
            if(mIsLcdScreenMode)
            {
                mHidInstance.SetLcdScreenModeBaudRate();
            }
            else
            {
                //使用LED指示灯控制
                mHidInstance.SetLedModeBaudRate();
            }
        }
        queueEvent(new Runnable()
        {
            @Override
            public void run()
            {
                final UVCCamera camera = new UVCCamera();
                camera.open(ctrlBlock);
                if (mPreviewSurface != null)
                {
                    mPreviewSurface.release();
                    mPreviewSurface = null;
                }

                try
                {
                    //camera.setPreviewSize(2688, 1944, UVCCamera.FRAME_FORMAT_MJPEG);
                    camera.setPreviewSize(2688, 1944, UVCCamera.FRAME_FORMAT_MJPEG);
                    //camera.setPreviewSize(1920, 1080, UVCCamera.FRAME_FORMAT_MJPEG);

                    Log.i(TAG, "mIFrameCallback...");

                    camera.setFrameCallback(mIFrameCallback, UVCCamera.PIXEL_FORMAT_RAW);
                }
                catch (final IllegalArgumentException e)
                {
                    Log.i(TAG, "xxxxxxxxxxxxxxxxxxxx...");
                    // fallback to YUV mode
                    try
                    {
                        camera.setPreviewSize(2592, 1944, UVCCamera.FRAME_FORMAT_YUYV);
                        //camera.setPreviewSize(2688, 1944, UVCCamera.FRAME_FORMAT_MJPEG);
                        //camera.setPreviewSize(1280, 720, UVCCamera.FRAME_FORMAT_MJPEG);
                        //camera.setFrameCallback(mIFrameCallback, UVCCamera.PIXEL_FORMAT_NV21);
                    }
                    catch (final IllegalArgumentException e1)
                    {
                        camera.destroy();
                        return;
                    }
                }

                final SurfaceTexture st = mUVCCameraView.getSurfaceTexture();

                if (st != null)
                {
                    mPreviewSurface = new Surface(st);

                    camera.setPreviewDisplay(mPreviewSurface);

                    camera.startPreview();
                }

                synchronized (mSync)
                {
                    mUVCCamera = camera;
                }



                System.arraycopy(UVCCamera.tag_para , 64 , distortionData, 0 , 2048);

                Log.i("distortionData" ,Bytes2HexString(distortionData , 100));

                //FingerAlgo.Init(distortionData);
                FingerAlgo.Init(distortionData);

                postResult(LS_ERROR_SUCCESS);

                mIsDeviceOpened = true;

                Log.i(TAG, "mIsDeviceOpened = true;");

                /////////////////////////

                PermissionCallback callback = new PermissionCallback();


            }
        }, 0);
    }

    private final USBMonitor.OnDeviceConnectListener mOnDeviceConnectListener = new USBMonitor.OnDeviceConnectListener()
    {
        @Override
        public void onAttach(final UsbDevice device)
        {
            Log.i(TAG, "onAttach..." + device.getProductId());
        }

        //设备连接后进行算法初始化检测
        @Override
        public void onConnect(final UsbDevice device, final USBMonitor.UsbControlBlock ctrlBlock, final boolean createNew)
        {
            Log.i(TAG, "onConnect...");
            openCameraDevice(ctrlBlock);
            mUsbControlBlock = ctrlBlock;
        }

        @Override
        public void onDisconnect(final UsbDevice device, final USBMonitor.UsbControlBlock ctrlBlock)
        {
            Log.i(TAG, "onDisconnect...");
            releaseCamera();
        }

        @Override
        public void onDettach(final UsbDevice device)
        {
            Log.i(TAG, "onDettach...");

            mIsDeviceOpened = false;
            mUsbControlBlock = null;
        }

        @Override
        public void onCancel(final UsbDevice device)
        {
            Log.i(TAG, "onDettach...");
            //设备授权被拒绝
            postResult(LS_ERROR_PERMISSION_DENIED);
        }
    };

    private synchronized void releaseCamera()
    {
        synchronized (mSync)
        {
            if (mUVCCamera != null)
            {
                try
                {
                    mUVCCamera.setStatusCallback(null);
                    mUVCCamera.setButtonCallback(null);
                    mUVCCamera.close();
                    mUVCCamera.destroy();
                }
                catch (final Exception e)
                {
                    //
                }
                finally
                {

                }

                mUVCCamera = null;
            }

            if (mPreviewSurface != null)
            {
                mPreviewSurface.release();
                mPreviewSurface = null;
            }

            if (mUSBMonitor != null)
            {
                mUSBMonitor.destroy();
                mUSBMonitor = null;
            }
        }
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

    //规定每段显示的长度
    private static int LOG_MAXLENGTH = 2000;

    public static void e(String TAG, String msg) {
        int strLength = msg.length();
        int start = 0;
        int end = LOG_MAXLENGTH;
        for (int i = 0; i < 10000; i++) {
            //剩下的文本还是大于规定长度则继续重复截取并输出
            if (strLength > end) {
                Log.i(TAG + i, msg.substring(start, end));
                start = end;
                end = end + LOG_MAXLENGTH;
            } else {
                Log.i(TAG, msg.substring(start, strLength));
                break;
            }
        }
    }

    private byte []     mCapturedFrame  = new byte[FINGER_WIDTH * FINGER_HEIGHT];
    private byte []     mCapturedBmp    = new byte[1078 + FINGER_WIDTH * FINGER_HEIGHT];
    private byte []     mCapturedBuffer = new byte[FINGER_WIDTH * FINGER_HEIGHT];
    //每一帧的回调函数
    private IFrameCallback mIFrameCallback = new IFrameCallback()
    {
        @Override
        public void onFrame(ByteBuffer frame)
        {
           /* try {
                Thread.sleep(500);
            } catch (InterruptedException e) {
                e.printStackTrace();
            }*/
            if (frame.limit() > 0)
            {
                frame.get(mYuvBuffer);

                if(FUFingerSDK.this.mPreviewCallback != null)
                {
                    //Log.i(TAG, "onFrame : " + frame.limit());

                    FUFingerSDK.this.postPreviewFinger(mYuvBuffer);
                }


                FingerAlgo.CorrDistortion(mYuvBuffer , ORG_WIDTH , ORG_HEIGHT , mCapturedBuffer, distortionData);

                if(mStartRoll)
                {
                    RawData data = new RawData();

                    data.w = FINGER_WIDTH;
                    data.h = FINGER_HEIGHT;

                    data.data = new byte[FINGER_WIDTH * FINGER_HEIGHT];

                    System.arraycopy(mCapturedBuffer , 0 , data.data , 0 , FINGER_WIDTH * FINGER_HEIGHT);

                    mRollFingerQueue.add(data);

                    Log.i(TAG, "onFrame : mRollFingerQueue.add(data);" + frame.limit());
                }

                //有线程需要采集一枚指纹
                if(mNeedCaptureFinger) //if(mNeedCaptureFinger.get() > 0)
                {
                    Log.i(TAG, "need capture");

                    System.arraycopy(mCapturedBuffer, 0, mCapturedFrame, 0, FINGER_WIDTH * FINGER_HEIGHT);
                    //System.arraycopy(greyBmp, 0, mCapturedBmp, 0, 1078 + FINGER_WIDTH * FINGER_HEIGHT);

                    mNeedCaptureFinger = false;

                    FUFingerSDK.this.mCaptureLatch.countDown();
                }
            }
        }
    };

    private void postPreviewFinger(byte [] greyBmp)
    {
        if (FUFingerSDK.this.mPreviewCallback != null)
        {
            FUFingerSDK.this.mUIHandler.post(new Runnable()
            {
                public void run()
                {
                    if (FUFingerSDK.this.mPreviewCallback != null )
                    {
                        FUFingerSDK.this.mPreviewCallback.onFrame(greyBmp);
                    }
                }
            });
        }
    }


    class  PermissionCallback implements IPermissionCallback
    {
        @Override
        public void execute(int result) //第一次打开设备需要授权的时候才会回调
        {
            if(HidDevice.ERR_SUCCESS == result)
            {
                //使用LCD 液晶屏控制
                if(mIsLcdScreenMode)
                {
                    mHidInstance.SetLcdScreenModeBaudRate();
                }
                else
                {
                    //使用LED指示灯控制
                    mHidInstance.SetLedModeBaudRate();
                }

                mIsDeviceOpened = true;

                return ;
            }


            Log.i(TAG, "if(HidDevice.ERR_SUCCESS !!!!!!= result)");


            mIsDeviceOpened = false;

            if(HidDevice.ERR_REFUSE == result)
            {
                return;
            }
        }
    }
}
