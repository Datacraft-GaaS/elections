package com.finger.demo;

public class JniFinger {
    public native String JniVersion();
    public native int JniSetImageArea(  int val  )  ;
    public native int JniSetImageHary( int val  ) ;
    public native int JniGetImageAreaHary(byte[] image, int w,int h  );
    public native int JniSetImagePore(  int val  ) ;
    public native int JniSetImageUniform( int val  ) ;
    public native int JniGetImagePore( byte[] image, int w,int h  );
    public native int JniGetImageUniform( byte[] image, int w,int h  ) ;

/*
    int Areaval = 40;
    int Haryval = 50000;
    int Poreval = 20;
    int Uniformval = 90;

    1.获取版本函数
    String JniFinger_JniVersion( ) ;
    参数：无
    返回：字符串

    2.设置指纹面域函数
    int JniSetImageArea(  int val  )  ;
    参数：int val；
    返回：上一次比对面域阈值
    注意：阈值0-100     默认：40

    3.设置指纹灰度函数
    int JniSetImageHary( int val  ) ;
    参数：int val；
    返回：上一次比对灰度阈值
    注意：阈值0-300000     默认：50000
    干手指：数值50000-90000  正常手指：90000-150000 湿手指：150000-300000

    4.获取指纹面域与灰度是否通过
    int JniGetImageAreaHary(byte[] image, int w,int h  )；
    参数：byte[] image 图像
    int w 图像宽度
    int h  图像高度
    返回：1表示通过  0表示不通过

    5.设置指纹毛孔数量
    int JniSetImagePore(  int val  ) ;
    参数：int val
    返回：上一次比对毛孔数量值    默认：20

    6.设置指纹图像均值分布
    int JniSetImageUniform( int val  ) ;
    参数：int val
    返回：上一次比对图像均值分布值    默认：90

    7.获取指纹毛孔数量
    int JniGetImagePore( byte[] image, int w,int h  );
    参数：byte[] image 图像
    int w 图像宽度
    int h  图像高度
    返回：1表示通过  0表示不通过

     6.获取指纹图像均值分布
    int JniGetImageUniform( byte[] image, int w,int h  ) ;
    参数：byte[] image 图像
    int w 图像宽度
    int h  图像高度
    返回：1表示通过  0表示不通过
*/


    static{
        System.loadLibrary("JniFinger");

    }
}
