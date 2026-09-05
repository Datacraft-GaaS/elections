package com.ZHIANG;

import com.ZHIANG.HostUsb;
import android.app.Activity;
import android.graphics.Bitmap;
import android.graphics.Bitmap.Config;
import android.util.Log;

import java.util.ArrayList;

public class FPAPI {
    static final String TAG = "FPAPI";
	public static final int VID = 0x28E9;//0x0483;//
	public static final int PID = 0x2018;// 0x5710;//
	private static HostUsb m_usbHost = null;
	private static int m_hUSB = 0;
	public static final int MSG_OPEN_DEVICE = 0x10;
	public static final int MSG_CLOSE_DEVICE = 0x11;
	public static final int MSG_BULK_TRANS_IN = 0x12;
	public static final int MSG_BULK_TRANS_OUT = 0x13;
	//****************************************************************************************************
	public static final int WIDTH  = 256;
	public static final int HEIGHT  = 360;
	public static final int IMAGE_SIZE = WIDTH*HEIGHT;
	//****************************************************************************************************
	public static final int FPINFO_STD_MAX_SIZE = 1024;
	public static final int DEF_FINGER_SCORE = 50;
	public static final int DEF_QUALITY_SCORE = 50;
	public static final int DEF_MATCH_SCORE = 45;
	//****************************************************************************************************
	public static final int TRUE = 1;
	public static final int FALSE = 0;
	//****************************************************************************************************
	private static Activity m_content = null;
	ArrayList<byte[]> fingerchars ;

	static 
	{
		try{
			System.loadLibrary("ZAZ_FpStdAPI");
		}
		catch(UnsatisfiedLinkError e) {
			Log.e("FPAPI","ZAZ_FpStdAPI",e);
		}
	}
	public FPAPI(Activity a) {
		m_content = a;
		fingerchars = new ArrayList<byte[]>();
	}

    //****************************************************************************************************
	private static int CallBack (int message, int notify, int param, Object data)
	{
		switch (message) {
		case MSG_OPEN_DEVICE:
			m_usbHost = new HostUsb ();
			if (!m_usbHost.AuthorizeDevice(m_content,VID, PID))
			{
				m_usbHost = null;
				return 0;
			}

		    m_hUSB = m_usbHost.OpenDeviceInterfaces();
			if (m_hUSB<0) {
				m_usbHost = null;
				return 0;
			}
			return m_hUSB;
		case MSG_CLOSE_DEVICE:
			if (m_usbHost != null) {
				m_usbHost.CloseDeviceInterface();
				m_hUSB = -1;
				m_usbHost = null;
			}
			return 1;
		case MSG_BULK_TRANS_IN:
			if (m_usbHost.USBBulkReceive((byte[])data,notify,param)) return notify;
			return 0;
		case MSG_BULK_TRANS_OUT:
			if (m_usbHost.USBBulkSend((byte[])data,notify,param)) return notify;
			return 0;
		}
		return 0;
	}
	public native long OpenDevice();// If successful, return handle of device, else 0.
	public native int CloseDevice(long device);//If successful, return 1, else 0
	public native int Calibration(long device, int mode);//If successful, return 1, else 0
	public native int GetImage(long device, byte[] image);//If successful, return 1, else 0
	public native int IsFinger(long device,byte[] image);//return percent value indicating that finger is placed on sensor(0~100).
	public native int GetImageQuality(long device,byte[] image);//return quality value(0~100) of fingerprint raw image.
	public native int GetNFIQuality(long device,byte[] image);//return NFI quality value(1~5) of fingerprint raw image.
	public native int CreateANSITemplate(long device,byte[] image, byte[] itemplate);//If this function successes, return size of template, else 0.
	public native int CreateISOTemplate(long device,byte[] image,  byte[] itemplate);//If this function successes, return size of template, else 0.
	public native int CompareTemplates(long device,byte[] itemplateToMatch, byte[] itemplateToMatched);//return similar match score(0~100) of two fingerprint templates.
 	public native int SearchingANSITemplates(long device, byte[] itemplateToSearch,//If successful, return index number of template searched inside template array,
		   	int numberOfDbTemplates, byte[] arrayOfDbTemplates, int scoreThreshold);//else -1.
	public native int SearchingISOTemplates(long device, byte[] itemplateToSearch,//If successful, return index number of template searched inside template array,
		   	int numberOfDbTemplates, byte[] arrayOfDbTemplates, int scoreThreshold);//else -1.
	public native int GetANSIImageRecord(long device,byte[] image, byte[] FIR);//If this function successes, return size of FIR, else 0.
	public native int GetISOImageRecord(long device,byte[] image, byte[] FIR);//If this function successes, return size of FIR, else 0.
	public native int  CompressToWSQImage (long device,byte[] rawImage, int w,int h,byte[] wsqImage);//return size of image compressed by WSQ
	public native int  UnCompressFromWSQImage (long device,byte[] wsqImage, int wsqSize, byte[] rawImage,int[] w,int[] h);//return size of uncompressed image
	//------------------------------------------------------------------------------------------------//

	private int isbus = 0;
	private int isonline = 0;



	public  long F_OpenDevice()
	{
		if(isbus!=0)return -101;
		isbus = 1;
		long ret = OpenDevice();
		if(ret != 0)
			isonline = 1;
		isbus = 0;
		return ret;
	}

	public int F_CloseDevice(long device)
	{
		if(isonline!=1)return -100;
		if(isbus!=0)return -101;
		isbus = 1;
		int ret = CloseDevice(device);
		isonline = 0;
		isbus = 0;
		return ret;
	}
	public int F_GetImage(long device, byte[] image)
	{
		if(isonline!=1)return -100;
		if(isbus!=0)return -101;
		isbus = 1;
		int ret = GetImage(device,image);
		isbus = 0;
		return ret;
	}
	public int F_GetImageQuality(long device, byte[] image)
	{
		if(isonline!=1)return -100;
		if(isbus!=0)return -101;
		isbus = 1;
		int ret = GetImageQuality(device,image);
		isbus = 0;
		return ret;
	}
	public int F_CreateTemplate(long device, byte[] image, byte[] itemplate)
	{
		if(isonline!=1)return -100;
		if(isbus!=0)return -101;
		isbus = 1;
		int ret = CreateANSITemplate(device,image,itemplate);
		isbus = 0;
		return ret;
	}
	public int F_CompareTemplates(long device, byte[] itemplateToMatch, byte[] itemplateToMatched)
	{
		if(isonline!=1)return -100;
		if(isbus!=0)return -101;
		isbus = 1;
		int ret = CompareTemplates(device,itemplateToMatch,itemplateToMatched);
		isbus = 0;
		return ret;
	}

    public int F_CompressToWSQImage (long device,byte[] rawImage, int w,int h,byte[] wsqImage)
    {
        return CompressToWSQImage(device,rawImage,w,h,wsqImage);
    }
    public int F_UnCompressFromWSQImage (long device,byte[] wsqImage, int wsqSize, byte[] rawImage,int[] w,int[] h)
    {
        return UnCompressFromWSQImage(device,wsqImage,wsqSize,rawImage,w,h);
    }




    public int F_SearchingISOTemplates(long device, byte[] itemplateToSearch,//If successful, return index number of template searched inside template array,
											int numberOfDbTemplates, byte[] arrayOfDbTemplates, int scoreThreshold)
	{
		if(isonline!=1)return -100;
		if(isbus!=0)return -101;
		isbus = 1;
		int ret = SearchingANSITemplates(device,itemplateToSearch,numberOfDbTemplates,arrayOfDbTemplates,scoreThreshold);
		isbus = 0;
		return ret;
	}

	public int Insertfingerchars(byte[] fpchar)
	{
		fingerchars.add(fpchar);
		return 1;
	}
	public int Clearfingerchars()
	{
		fingerchars.clear();
		return 1;
	}


	public ArrayList<byte[]> searchfingerchars()
	{

		return fingerchars;

	}
	public int getfingercharsSize(){
		return fingerchars.size();
	}

}
