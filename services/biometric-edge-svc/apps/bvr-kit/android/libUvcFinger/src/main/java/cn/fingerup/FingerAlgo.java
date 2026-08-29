package cn.fingerup;

public class FingerAlgo
{
    static
    {
        System.loadLibrary("fingeralgo");
    }

    public static int DoSplit(byte [] pImgBuf, int nImgW, int nImgH, int nSplitW, int nSplitH, FPSPLIT_INFO[] pInfo)
    {
        byte [] outInfo = new byte[nSplitW * nSplitH * 12];

        int result = DoSplit(pImgBuf , nImgW , nImgH , nSplitW , nSplitH , outInfo);

        if(result <= 0)
        {
            return result;
        }

        int offset = 0;
        //拆分成功从byte[] 中解析出来具体信息
        for(int i = 0 ; i < result ; ++i)
        {
            pInfo[i].x = (int) ((outInfo[offset] & 0xFF)
                    | ((outInfo[offset+1] & 0xFF)<<8)
                    | ((outInfo[offset+2] & 0xFF)<<16)
                    | ((outInfo[offset+3] & 0xFF)<<24));


            offset += 4;

            pInfo[i].y = (int) ((outInfo[offset] & 0xFF)
                    | ((outInfo[offset+1] & 0xFF)<<8)
                    | ((outInfo[offset+2] & 0xFF)<<16)
                    | ((outInfo[offset+3] & 0xFF)<<24));

            offset += 4;

            pInfo[i].angle = (int) ((outInfo[offset] & 0xFF)
                    | ((outInfo[offset+1] & 0xFF)<<8)
                    | ((outInfo[offset+2] & 0xFF)<<16)
                    | ((outInfo[offset+3] & 0xFF)<<24));

            offset += 4;

            System.arraycopy(outInfo , offset , pInfo[i].pOutBuf , 0 , nSplitW * nSplitH);

            offset += nSplitW * nSplitH;
        }

        return result;
    }


    public static native int Init(byte [] distortionData);
    public static native int Release();

    public static native int GetRollFinger(byte [] inArray, int nImgW, int nImgH , byte [] outArray, int nNewImgW, int nNewImgH);
    public static native int GetFlatFinger(byte [] inArray, int nImgW, int nImgH , byte [] outArray, int nNewImgW, int nNewImgH);

    //四指拆分接口
    public static native int DoSplit(byte [] pImgBuf, int nImgW, int nImgH, int nSplitW, int nSplitH, byte [] pInfo);

    //畸变矫正接口
    public static native int CorrDistortion(byte [] pImgBuf, int nImgW, int nImgH, byte [] pOutBuf , byte [] distortionData);

    //指纹拼接函数接口

    public static native int MOSAIC_Init();
    public static native int MOSAIC_Close();
    public static native int MOSAIC_Stop();
    public static native int MOSAIC_IsFinger(byte [] pImgBuf, int nImgW, int nImgH);
    public static native int MOSAIC_Start(byte [] pImgBuf, int nImgW, int nImgH);
    public static native int MOSAIC_DoMosaic(byte [] pImgBuf, int nImgW, int nImgH);
    public static native int MOSAIC_FingerArea(byte [] pImgBuf, int nImgW, int nImgH);


    public static native int MOSAIC_IsSupportIdentifyFinger();
    public static native int MOSAIC_IsSupportRollCap();
    public static native int MOSAIC_IsSupportImageQuality();
    public static native int MOSAIC_IsSupportFingerQuality();
    public static native int MOSAIC_IsSupportImageEnhance();

    public static native int MOSAIC_SetRollMode(int nRollMode);

    public static native int MOSAIC_ImageQuality(byte [] pDataBuf, int nWidth, int nHeight);
    public static native int MOSAIC_FingerQuality(byte [] pDataBuf, int nWidth, int nHeight);
    public static native int MOSAIC_ImageEnhance(byte [] pSrcBuf, int nWidth, int nHeight, byte [] pTargetImg);

    //public static native int MOSAIC_GetErrorInfo(int nErrorNo, char [] pszErrorInfo);
    public static native int MOSAIC_GetVersion();
    //public static native int MOSAIC_GetDesc(char [] pszDesc);
    public static native int SaveBmp(String name, byte[] data, int w, int h);

}
