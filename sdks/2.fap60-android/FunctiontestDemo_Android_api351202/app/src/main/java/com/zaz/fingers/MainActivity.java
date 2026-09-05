package com.zaz.fingers;

import androidx.annotation.NonNull;
import androidx.appcompat.app.ActionBar;
import androidx.appcompat.app.AppCompatActivity;
import androidx.core.app.ActivityCompat;
import androidx.core.content.ContextCompat;

import android.Manifest;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.graphics.Bitmap;
import android.graphics.BitmapFactory;
import android.os.Bundle;
import android.os.Environment;
import android.util.Log;
import android.view.View;
import android.view.WindowManager;
import android.widget.Button;
import android.widget.CheckBox;
import android.widget.CompoundButton;
import android.widget.ImageView;
import android.widget.RadioGroup;
import android.widget.TextView;
import android.widget.Toast;

import com.ZHIANG.FPAPI;
import com.finger.demo.JniFinger;

import java.io.ByteArrayOutputStream;
import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

import cn.fingerup.FPSPLIT_INFO;
import cn.fingerup.FUFingerSDK;
import cn.fingerup.FingerAlgo;
import cn.fingerup.OnConnectListener;

public class MainActivity extends AppCompatActivity {
    String[] permissions = new String[]{Manifest.permission.CAMERA, Manifest.permission.WRITE_EXTERNAL_STORAGE};
    List<String> mPermissionList = new ArrayList<>();
    private static final int PERMISSION_REQUEST = 1;
    private Button mBtnInit, mBtndisconnBtn, mbtnsavebmp, mbtnenrollfinger, mbtnsearchfinger,
            mbtnenrolllist, mbtnempty, mbtnsingle, mbtnroll, mbtnsplitleftfp, mbtnsplitrightfp, mbtnsplittwofp,mcancel;
    private RadioGroup typeradio;
    private TextView tvmsg;
    private ImageView imageView;
    private CheckBox matchcheckBox, LcdcheckBox;
    private FingerAlgo fingerAlgo;
    private FPAPI zaclient;
    private Button[] btnarr = new Button[11];
    FUFingerSDK mFufingerSDK;
    JniFinger jniFinger;
    private long usbdevice = 0;
    private boolean isonline = false;
    private boolean iscancel = false;

    Bitmap bmpdata;
    private byte[] rawdata;
    private byte[] rawdatas;
    boolean nullimage=true;
    private int rord=0;
    public final static int FingerQuality = 40;
    SoundPoolPlayer mSoundPlayer = null;
    boolean LCD = false;

    public static byte[] msplitfinger=new byte[1600*1500];
    public static byte[] mfinger=new byte[800*750];
    public static byte[] mdfinger=new byte[300*400];

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);
        //获得ActionBar实例
        ActionBar supportActionBar = getSupportActionBar();
        if (supportActionBar != null) {
            //调用hide()方法将标题栏隐藏起来
            supportActionBar.hide();
        }
        getWindow().setFlags(WindowManager.LayoutParams.FLAG_FULLSCREEN, WindowManager.LayoutParams.FLAG_FULLSCREEN);
        checkPermission();
        typeradio = findViewById(R.id.type);
        mBtnInit = findViewById(R.id.connBtn);
        mbtnroll = findViewById(R.id.rollBtn);
        mBtndisconnBtn = findViewById(R.id.disconnBtn);
        mbtnsavebmp = findViewById(R.id.saveimg);
        mbtnenrollfinger = findViewById(R.id.enrollfingerBtn);
        mbtnsearchfinger = findViewById(R.id.verifyfingerBtn);
        mbtnenrolllist = findViewById(R.id.getfingerlist);
        mbtnempty = findViewById(R.id.getemqtyBtn);

        imageView = findViewById(R.id.imgv);
        tvmsg = findViewById(R.id.hintTv);
        mbtnsingle = findViewById(R.id.singleBtn);
        mbtnsplitleftfp = findViewById(R.id.splitleftfpBtn);
        mbtnsplitrightfp = findViewById(R.id.splitrightfpBtn);
        mbtnsplittwofp = findViewById(R.id.twothumbBtn);
        matchcheckBox = findViewById(R.id.matchcheckbox);
        LcdcheckBox = findViewById(R.id.LCDcheckbox);
        mcancel = findViewById(R.id.cancel_button);
        btnarr[0] = mBtnInit;
        btnarr[1] = mBtndisconnBtn;
        btnarr[2] = mbtnenrollfinger;
        btnarr[3] = mbtnsearchfinger;
        btnarr[4] = mbtnenrolllist;
        btnarr[5] = mbtnempty;
        btnarr[6] = mbtnsingle;
        btnarr[7] = mbtnsplitleftfp;
        btnarr[8] = mbtnsplitrightfp;
        btnarr[9] = mbtnroll;
        btnarr[10] = mbtnsplittwofp;
        zaclient = new FPAPI(this);
        fingerAlgo = new FingerAlgo();
        jniFinger = new JniFinger();
        mSoundPlayer = new SoundPoolPlayer(this);
        mFufingerSDK = new FUFingerSDK(this);
        mFufingerSDK.setConnectListener(new OnConnectListener() {
            @Override
            public void onConnected() {
                Sleep(200);
                mFufingerSDK.LIVESCAN_Beep(1);
                showmsg(getString(R.string.DeviceConnected));
                fingerAlgo.MOSAIC_Init();
                isonline = true;
                SetButton(true);
                LCD = LcdcheckBox.isChecked();
            }

            @Override
            public void onConnectFailed(int code) {
                showmsg(getString(R.string.openfail) + " code:" + code);
                SetButton(true);
                matchcheckBox.setEnabled(true);
                LcdcheckBox.setEnabled(true);
            }
        });
        matchcheckBox.setOnCheckedChangeListener(new CompoundButton.OnCheckedChangeListener() {
            @Override
            public void onCheckedChanged(CompoundButton buttonView, boolean isChecked) {
                if (isChecked) {
                    mbtnsearchfinger.setVisibility(View.VISIBLE);
                    mbtnenrollfinger.setVisibility(View.VISIBLE);
                    mbtnenrolllist.setVisibility(View.VISIBLE);
                    mbtnempty.setVisibility(View.VISIBLE);
                } else {
                    mbtnsearchfinger.setVisibility(View.GONE);
                    mbtnenrollfinger.setVisibility(View.GONE);
                    mbtnenrolllist.setVisibility(View.GONE);
                    mbtnempty.setVisibility(View.GONE);
                }
            }
        });


        mBtnInit.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {

                connBtnClick();//打开设备
            }
        });
        mcancel.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                iscancel = true;
            }
        });
        mBtndisconnBtn.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                isonline = false;
                SetButton(true);
                showFingerImg(null, 0, 0);
                if (mFufingerSDK.LIVESCAN_Close() == 1) {
                    showmsg(getString(R.string.closeok));
                    matchcheckBox.setEnabled(true);
                    LcdcheckBox.setEnabled(true);
                    showmsg(getString(R.string.closeok));
                } else {
                    showmsg(getString(R.string.closefail));
                }
            }
        });
        typeradio.setOnCheckedChangeListener(new RadioGroup.OnCheckedChangeListener() {
            @Override
            public void onCheckedChanged(RadioGroup group, int checkedId) {
                if (!isonline) {
                    showmsg(getString(R.string.firstopen));
                    return;
                }
                if (checkedId == R.id.normal) {
                    mFufingerSDK.LIVESCAN_SetFingerDryWet(4);
                } else if (checkedId == R.id.wet) {
                    mFufingerSDK.LIVESCAN_SetFingerDryWet(3);
                } else if (checkedId == R.id.dry) {
                    mFufingerSDK.LIVESCAN_SetFingerDryWet(5);
                }
            }

        });
        mbtnsingle.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                if (!isonline) {
                    showmsg(getString(R.string.firstopen));
                    return;
                }
                Runnable r = new Runnable() {
                    public void run() {
                        SetButton(false);
                        mSoundPlayer.playShortResource(5);
                        if (LCD)
                            mFufingerSDK.LcdImage(FUFingerSDK.LEFT_THUMB_FINGER);
                        else
                            mFufingerSDK.LedLight(FUFingerSDK.LEFT_THUMB_FINGER);
                        iscancel = false;
                        singleBtnClick();
                        SetButton(true);
                    }
                };
                Thread s = new Thread(r);
                s.start();
            }
        });
        imageView.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                if(nullimage){
                    return;
                }
                Intent intent = new Intent(MainActivity.this,showPhoto.class);
                if(rord==1){
                    intent.putExtra("photoPath","roll");
                } else if(rord==2){
                    intent.putExtra("photoPath","df");
                }else {
                    intent.putExtra("photoPath","imagesplit");
                }

                startActivity(intent);
            }
        });
        mbtnsplitleftfp.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                if (!isonline) {
                    showmsg(getString(R.string.firstopen));
                    return;
                }
                Runnable r = new Runnable() {
                    public void run() {
                        SetButton(false);
                        showFingerImg(null, 0, 0);
                        mSoundPlayer.playShortResource(2);
                        if (LCD)
                            mFufingerSDK.LcdImage(FUFingerSDK.LEFT_FOUR_FINGER);
                        else
                            mFufingerSDK.LedLight(FUFingerSDK.LEFT_FOUR_FINGER);
                        iscancel = false;
                        SplitfingersBtnClick(1);
                        SetButton(true);
                    }
                };
                Thread s = new Thread(r);
                s.start();
            }
        });
        mbtnsplitrightfp.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                if (!isonline) {
                    showmsg(getString(R.string.firstopen));
                    return;
                }
                Runnable r = new Runnable() {
                    public void run() {
                        SetButton(false);
                        showFingerImg(null, 0, 0);
                        mSoundPlayer.playShortResource(3);
                        if (LCD)
                            mFufingerSDK.LcdImage(FUFingerSDK.RIGHT_FOUR_FINGER);
                        else
                            mFufingerSDK.LedLight(FUFingerSDK.RIGHT_FOUR_FINGER);
                        iscancel = false;
                        SplitfingersBtnClick(2);
                        SetButton(true);
                    }
                };
                Thread s = new Thread(r);
                s.start();
            }
        });

        mbtnsplittwofp.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                if (!isonline) {
                    showmsg(getString(R.string.firstopen));
                    return;
                }
                Runnable r = new Runnable() {
                    public void run() {
                        SetButton(false);
                        showFingerImg(null, 0, 0);
                        mSoundPlayer.playShortResource(1);
                        if (LCD)
                            mFufingerSDK.LcdImage(FUFingerSDK.TWO_THUMB_FINGER);
                        else
                            mFufingerSDK.LedLight(FUFingerSDK.TWO_THUMB_FINGER);
                        iscancel = false;
                        SplitfingersBtnClick(3);
                        SetButton(true);
                    }
                };
                Thread s = new Thread(r);
                s.start();
            }
        });

        mbtnroll.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                if (!isonline) {
                    showmsg(getString(R.string.firstopen));
                    return;
                }
                Runnable r = new Runnable() {
                    public void run() {
                        SetButton(false);
                        mSoundPlayer.playShortResource(5);
                        if (LCD)
                            mFufingerSDK.LcdImage(FUFingerSDK.LEFT_THUMB_FINGER);
                        else
                            mFufingerSDK.LedLight(FUFingerSDK.LEFT_THUMB_FINGER);
                        iscancel = false;
                        Collectrollingfinger();
                        SetButton(true);
                    }
                };
                Thread s = new Thread(r);
                s.start();
            }
        });

        mbtnsavebmp.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                if (!isonline) {
                    showmsg(getString(R.string.firstopen));
                    return;
                }
                savebmpBtnClick();
            }
        });

        mbtnenrollfinger.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                if (!isonline) {
                    showmsg(getString(R.string.firstopen));
                    return;
                }

                Runnable r = new Runnable() {
                    public void run() {
                        usbdevice = usbdevice == 0 ? zaclient.F_OpenDevice() : usbdevice;
                        if (usbdevice == 0) {
                            showmsg(getString(R.string.initfinger));
                            return;
                        }
                        SetButton(false);
                        iscancel = false;
                        enrollfingerBtnClick();
                        SetButton(true);
                    }
                };
                Thread s = new Thread(r);
                s.start();
            }
        });

        mbtnsearchfinger.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                if (!isonline) {
                    showmsg(getString(R.string.firstopen));
                    return;
                }
                Runnable r = new Runnable() {
                    public void run() {
                        usbdevice = usbdevice == 0 ? zaclient.F_OpenDevice() : usbdevice;
                        if (usbdevice == 0) {
                            showmsg(getString(R.string.initfinger));
                            return;
                        }
                        SetButton(false);
                        iscancel = false;
                        searchfingerBtnClick();
                        SetButton(true);
                    }
                };
                Thread s = new Thread(r);
                s.start();
            }
        });

        mbtnenrolllist.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                if (!isonline) {
                    showmsg(getString(R.string.firstopen));
                    return;
                }
                Runnable r = new Runnable() {
                    public void run() {
                        GetEnrollListBtnClick();
                    }
                };
                Thread s = new Thread(r);
                s.start();

            }
        });
        mbtnempty.setOnClickListener(new View.OnClickListener() {
            public void onClick(View v) {
                if (!isonline) {
                    showmsg(getString(R.string.firstopen));
                    return;
                }
                Runnable r = new Runnable() {
                    public void run() {
                        emptyBtnClick();
                    }
                };
                Thread s = new Thread(r);
                s.start();

            }
        });



    }

    private void connBtnClick() {
        SetButton(false);
        matchcheckBox.setEnabled(false);
        LcdcheckBox.setEnabled(false);
        Runnable r = new Runnable() {
            public void run() {
                int ret =  mFufingerSDK.LIVESCAN_Init(LcdcheckBox.isChecked());
                //if(ret == 1)
                {
                    //success
                    Log.e("TAG","LIVESCAN_Init SUCCESS");
                    usbdevice = zaclient.F_OpenDevice();
                    if(usbdevice!= 0)
                    {
                        Log.e("TAG","F_OpenDevice SUCCESS");
                        //success
                    }
                }
            }
        };
        Thread s = new Thread(r);
        s.start();


        mbtnsearchfinger.setVisibility(View.VISIBLE);
        mbtnenrollfinger.setVisibility(View.VISIBLE);
        mbtnenrolllist.setVisibility(View.VISIBLE);
        mbtnempty.setVisibility(View.VISIBLE);



    }

    private void singleBtnClick() {
        showmsg(getString(R.string.start));
        long starttime = System.currentTimeMillis();
        mFufingerSDK.LIVESCAN_Beep(1);
        int count = 0;
        while (isonline && (System.currentTimeMillis() - starttime) < 10000) {
            byte[] rawdata = new byte[FUFingerSDK.FLAT_FINGER_WIDTH * FUFingerSDK.FLAT_FINGER_HEIGHT];
            int nResult = mFufingerSDK.LIVESCAN_GetFlatFPRawData(rawdata, FUFingerSDK.FLAT_FINGER_WIDTH, FUFingerSDK.FLAT_FINGER_HEIGHT);
            rord=2;
            mdfinger = rawdata;
            showFingerImg(rawdata, FUFingerSDK.FLAT_FINGER_WIDTH, FUFingerSDK.FLAT_FINGER_HEIGHT);
            if (nResult != 1) {
                showmsg(getString(R.string.getimageerr) + nResult);
                if (LCD)
                    mFufingerSDK.LcdImage(0);
                else
                    mFufingerSDK.LedLight(0);
                return;
            }
            nResult = FingerAlgo.MOSAIC_FingerQuality(rawdata, FUFingerSDK.FLAT_FINGER_WIDTH, FUFingerSDK.FLAT_FINGER_HEIGHT);
            if (nResult > FingerQuality) {
                count++;
                if (count < 5) continue;
                count = 0;
                showmsg(getString(R.string.CaptureSuccess) + nResult);
                mFufingerSDK.LIVESCAN_Beep(2);
                mSoundPlayer.playShortResource(15);
                if (LCD)
                    mFufingerSDK.LcdImage(FUFingerSDK.FINGER_SUCCESS);
                else
                    mFufingerSDK.LedLight(0);
                return;
            }
        }
        showmsg(getString(R.string.timeout));
        mSoundPlayer.playShortResource(16);
        if (LCD)
            mFufingerSDK.LcdImage(FUFingerSDK.FINGER_FAIL);
        else
            mFufingerSDK.LedLight(0);
    }


    private void SplitfingersBtnClick(int type) {
        rord = 3;
        nullimage=true;
        showmsg(getString(R.string.start));
        FPSPLIT_INFO[] info = new FPSPLIT_INFO[10];
        int SPLIT_IMG_WIDTH = 256;
        int SPLIT_IMG_HEIGHT = 360;
        int IMG_WIDTH = 1600;
        int IMG_HEIGHT = 1500;
        int count = 0;
        int success_nums=0;
        boolean getfingertype = false;
        int[] a = new int[4];
        int[] b = new int[4];

        for (int i = 0; i < 10; ++i) {
            info[i] = new FPSPLIT_INFO();
            info[i].pOutBuf = new byte[SPLIT_IMG_WIDTH * SPLIT_IMG_HEIGHT];
        }
        int fingernum = type == 3 ? 2 : 4;
        mFufingerSDK.LIVESCAN_Beep(1);
        long starttime = System.currentTimeMillis();
        while (!iscancel && isonline && (System.currentTimeMillis() - starttime) < 10000) {
            byte[] rawdata = new byte[IMG_WIDTH * IMG_HEIGHT];
            int ret = mFufingerSDK.captureFinger(100, rawdata, null);

            if (ret == 1) {
                msplitfinger=rawdata;
                showFingerImg(rawdata, IMG_WIDTH, IMG_HEIGHT);
            } else {
                continue;
            }

            int fingerCount = FingerAlgo.DoSplit(rawdata, IMG_WIDTH, IMG_HEIGHT, SPLIT_IMG_WIDTH, SPLIT_IMG_HEIGHT, info);

            //showmsg(getString(R.string.fingernum) + fingerCount);
            if (fingerCount == fingernum) {
                count++;
                if (count < 5) continue;
                count = 0;
                boolean isQualityGood = true;
                int min = 1000;
                for (int i = 0; i < fingerCount; ++i) {
                    a[i] = info[i].y;
                    min = Math.min(min, a[i]);

                    ret = FingerAlgo.MOSAIC_FingerQuality(info[i].pOutBuf, SPLIT_IMG_WIDTH, SPLIT_IMG_HEIGHT);
                    if (ret < FingerQuality) {
                        isQualityGood = false;
                        break;
                    }
                }
                if(isQualityGood)
                {
                    success_nums++;
                }
                else{
                    success_nums=0;
                    continue;
                }
//                if((type ==1 && a[0] > a[3] && min == a[2]) ||(type ==2 && a[0] < a[3] && min == a[1])||type == 3 ){
//                    getfingertype = true;
//                }else{
//                    if(type==1){
//                        showmsg(getString(R.string.leftfinger));
//                    }else {
//                        showmsg(getString(R.string.rightfinger));
//                    }
//                }
                if (isQualityGood && success_nums==3) {
                    showmsg(getString(R.string.CaptureSuccess) + ret);
                    mFufingerSDK.LIVESCAN_Beep(2);
                    mSoundPlayer.playShortResource(15);
                    if (LCD) {
                        if (type == 1) {
                            mFufingerSDK.LcdImage(FUFingerSDK.LEFT_FOUR_FINGER_SUCCESS);
                        } else if (type == 2) {
                            mFufingerSDK.LcdImage(FUFingerSDK.RIGHT_FOUR_FINGER_SUCCESS);
                        } else {
                            mFufingerSDK.LcdImage(FUFingerSDK.TWO_THUMB_FINGER_SUCCESS);
                        }
                    } else
                        mFufingerSDK.LedLight(0);
                    nullimage=false;
                    return;
                }
            }

        }
        nullimage=false;
        showmsg(getString(R.string.timeout));
        mSoundPlayer.playShortResource(16);
        if (LCD) {
            if (type == 1) {
                mFufingerSDK.LcdImage(FUFingerSDK.LEFT_FOUR_FINGER_FAIL);
            } else if (type == 2) {
                mFufingerSDK.LcdImage(FUFingerSDK.RIGHT_FOUR_FINGER_FAIL);
            } else {
                mFufingerSDK.LcdImage(FUFingerSDK.TWO_THUMB_FINGER_FAIL);
            }
        } else
            mFufingerSDK.LedLight(0);
    }

    private void Collectrollingfinger() {
        nullimage=true;
        int ROLL_IMG_WIDTH = 800;
        int ROLL_IMG_HEIGHT = 750;
        byte[] rawData = new byte[ROLL_IMG_WIDTH * ROLL_IMG_HEIGHT];
        byte[] g_pMosicBuf = new byte[ROLL_IMG_WIDTH * ROLL_IMG_HEIGHT];
        Arrays.fill(g_pMosicBuf, (byte) 0XFF);
        mFufingerSDK.LIVESCAN_Beep(1);
        FUFingerSDK.mStartRoll = true;
        int g_bFinger = 0;
        long starttime = System.currentTimeMillis();
        while (!iscancel && isonline && (System.currentTimeMillis() - starttime) < 10000) {
            int nResult = mFufingerSDK.LIVESCAN_GetRollFPRawData(rawData, ROLL_IMG_WIDTH, ROLL_IMG_HEIGHT);
            if (mFufingerSDK.LS_ERROR_SUCCESS != nResult) {
                continue;
            }
            if (g_bFinger <= 0) {
                g_bFinger = FingerAlgo.MOSAIC_IsFinger(rawData, ROLL_IMG_WIDTH, ROLL_IMG_HEIGHT);

                if (g_bFinger <= 0) {
                    continue;
                }

                FingerAlgo.MOSAIC_Start(g_pMosicBuf, ROLL_IMG_WIDTH, ROLL_IMG_HEIGHT);
            }
            int nRet = FingerAlgo.MOSAIC_DoMosaic(rawData, ROLL_IMG_WIDTH, ROLL_IMG_HEIGHT);
            if (nRet > 0) //继续拼接
            {
                showFingerImg(g_pMosicBuf, ROLL_IMG_WIDTH, ROLL_IMG_HEIGHT);
                continue;
            }

            g_bFinger = 0;

            FingerAlgo.MOSAIC_Stop();

            FUFingerSDK.mStartRoll = false;

            mFufingerSDK.mRollFingerQueue.clear();
            mFufingerSDK.LIVESCAN_Beep(2);
            if (nRet < 0) {
                showmsg(getString(R.string.getimageerr) + nRet);
                if (LCD)
                    mFufingerSDK.LcdImage(0);
                else
                    mFufingerSDK.LedLight(0);
                return;
            } else {
                rord=1;
                mfinger=g_pMosicBuf;
                nResult = FingerAlgo.MOSAIC_FingerQuality(g_pMosicBuf, ROLL_IMG_WIDTH, ROLL_IMG_HEIGHT);
                showmsg(getString(R.string.CaptureSuccess) + nResult);
                mFufingerSDK.LIVESCAN_Beep(2);
                mSoundPlayer.playShortResource(15);
                if (LCD) {
                    mFufingerSDK.LcdImage(FUFingerSDK.FINGER_SUCCESS);
                } else
                    mFufingerSDK.LedLight(0);
                nullimage=false;
                return;
            }

        }
        nullimage=false;
        showmsg(getString(R.string.timeout));
        mSoundPlayer.playShortResource(16);
        if (LCD)
            mFufingerSDK.LcdImage(FUFingerSDK.FINGER_FAIL);
        else
            mFufingerSDK.LedLight(0);
    }


    private void enrollfingerBtnClick() {
        rord = 3;

        nullimage=true;
        showmsg(getString(R.string.start));
        FPSPLIT_INFO[] info = new FPSPLIT_INFO[10];
        int SPLIT_IMG_WIDTH = 256;
        int SPLIT_IMG_HEIGHT = 360;
        int IMG_WIDTH = 1600;
        int IMG_HEIGHT = 1500;
        int count = 0;
        int success_nums=0;
        for (int i = 0; i < 10; ++i) {
            info[i] = new FPSPLIT_INFO();
            info[i].pOutBuf = new byte[SPLIT_IMG_WIDTH * SPLIT_IMG_HEIGHT];
        }
        mFufingerSDK.LIVESCAN_Beep(1);
        long starttime = System.currentTimeMillis();
        while (!iscancel && isonline && (System.currentTimeMillis() - starttime) < 10000) {
            byte[] rawdata = new byte[IMG_WIDTH * IMG_HEIGHT];
            int ret = mFufingerSDK.captureFinger(100, rawdata, null);
            if (ret == 1) {
                msplitfinger=rawdata;
                showFingerImg(rawdata, IMG_WIDTH, IMG_HEIGHT);
            } else {
                continue;
            }

            int fingerCount = FingerAlgo.DoSplit(rawdata, IMG_WIDTH, IMG_HEIGHT, SPLIT_IMG_WIDTH, SPLIT_IMG_HEIGHT, info);
            showmsg(getString(R.string.fingernum) + fingerCount);

            if (fingerCount ==4) {
                count++;
                if (count < 5) continue;
                count = 0;
                boolean isQualityGood = true;
                for (int i = 0; i < fingerCount; ++i) {
                    ret = FingerAlgo.MOSAIC_FingerQuality(info[i].pOutBuf, SPLIT_IMG_WIDTH, SPLIT_IMG_HEIGHT);
                    if (ret < FingerQuality) {
                        isQualityGood = false;
                        break;
                    }
                }
                if(isQualityGood)
                {
                    success_nums++;
                }
                else{
                    success_nums=0;
                    continue;
                }
                if (isQualityGood && success_nums==3) {
                    byte[] fparr = new byte[1024 * fingerCount];
                    boolean issuccess = true;
                    for (int i = 0; i < fingerCount; i++) {
                        byte[] fp = new byte[1024];
                        if (zaclient.CreateISOTemplate(usbdevice, info[i].pOutBuf, fp) > 0) {
                            System.arraycopy(fp, 0, fparr, i * 1024, 1024);
                        } else {
                            issuccess = false;
                            break;
                        }
                    }
                    if (issuccess) {
                        mFufingerSDK.LIVESCAN_Beep(2);
                        zaclient.Insertfingerchars(fparr);
                        showmsg(getString(R.string.Enrollok) + (zaclient.getfingercharsSize() - 1));
                        if (LCD)
                            mFufingerSDK.LcdImage(FUFingerSDK.FINGER_SUCCESS);
                        else
                            mFufingerSDK.LedLight(0);
                        nullimage = false;
                        return;
                    }
                }
            }
            else {
                count=0;
            }

        }
        nullimage = false;
        showmsg(getString(R.string.timeout));
        mSoundPlayer.playShortResource(16);
        if (LCD) {
            mFufingerSDK.LcdImage(FUFingerSDK.FINGER_FAIL);
        } else
            mFufingerSDK.LedLight(0);
    }

    private void searchfingerBtnClick() {
        rord = 3;
        nullimage = true;
        showmsg(getString(R.string.start));
        FPSPLIT_INFO[] info = new FPSPLIT_INFO[10];
        int SPLIT_IMG_WIDTH = 256;
        int SPLIT_IMG_HEIGHT = 360;
        int IMG_WIDTH = 1600;
        int IMG_HEIGHT = 1500;
        int mcount = 0;
        for (int i = 0; i < 10; ++i) {
            info[i] = new FPSPLIT_INFO();
            info[i].pOutBuf = new byte[SPLIT_IMG_WIDTH * SPLIT_IMG_HEIGHT];
        }
        mFufingerSDK.LIVESCAN_Beep(1);
        long starttime = System.currentTimeMillis();
        while (!iscancel && isonline && (System.currentTimeMillis() - starttime) < 10000) {
            byte[] rawdata = new byte[IMG_WIDTH * IMG_HEIGHT];
            int ret = mFufingerSDK.captureFinger(100, rawdata, null);
            if (ret == 1) {
                msplitfinger=rawdata;
                showFingerImg(rawdata, IMG_WIDTH, IMG_HEIGHT);
            } else {
                continue;
            }

            int fingerCount = FingerAlgo.DoSplit(rawdata, IMG_WIDTH, IMG_HEIGHT, SPLIT_IMG_WIDTH, SPLIT_IMG_HEIGHT, info);
            showmsg(getString(R.string.fingernum) + fingerCount);
            if (fingerCount == 4) {
                mcount++;
                if (mcount < 5) continue;
                mcount = 0;
                boolean isQualityGood = true;
                for (int i = 0; i < fingerCount; ++i) {
                    ret = FingerAlgo.MOSAIC_FingerQuality(info[i].pOutBuf, SPLIT_IMG_WIDTH, SPLIT_IMG_HEIGHT);
                    if (ret < FingerQuality) {
                        isQualityGood = false;
                        break;
                    }
                }
                if (isQualityGood) {
                    byte[] fparr = new byte[1024 * fingerCount];
                    boolean issuccess = true;
                    for (int i = 0; i < fingerCount; i++) {
                        byte[] fp = new byte[1024];
                        if (zaclient.CreateISOTemplate(usbdevice, info[i].pOutBuf, fp) > 0) {
                            System.arraycopy(fp, 0, fparr, i * 1024, 1024);
                        } else {
                            issuccess = false;
                            break;
                        }
                    }
                    if (issuccess) {
                        boolean matchok = false;
                        int id = -1;
                        for (int i = 0; i < zaclient.getfingercharsSize(); i++) {
                            if (zaclient.searchfingerchars().get(i).length / 1024 != fingerCount) {
                                continue;
                            }
                            int count = 0;
                            for (int j = 0; j < fingerCount; j++) {
                                byte[] fp1 = new byte[1024];
                                byte[] fp2 = new byte[1024];
                                System.arraycopy(zaclient.searchfingerchars().get(i), j * 1024, fp1, 0, 1024);
                                System.arraycopy(fparr, j * 1024, fp2, 0, 1024);
                                if (zaclient.CompareTemplates(usbdevice, fp1, fp2) > 40) {
                                    count++;
                                }
                            }
                            if (count == fingerCount) {
                                matchok = true;
                                id = i;
                                break;
                            }
                        }
                        if (matchok) {
                            mFufingerSDK.LIVESCAN_Beep(2);
                            showmsg(getString(R.string.searchok) + id);
                            if (LCD)
                                mFufingerSDK.LcdImage(FUFingerSDK.FINGER_SUCCESS);
                            else
                                mFufingerSDK.LedLight(0);
                            nullimage = false;
                            return;
                        } else {
                            showmsg(getString(R.string.searchfail));
                            if (LCD) {
                                mFufingerSDK.LcdImage(FUFingerSDK.FINGER_FAIL);
                            } else
                                mFufingerSDK.LedLight(0);
                            nullimage = false;
                            return;
                        }

                    }
                }
            }

        }
        nullimage = false;
        showmsg(getString(R.string.timeout));
        mSoundPlayer.playShortResource(16);
        if (LCD) {
            mFufingerSDK.LcdImage(FUFingerSDK.FINGER_FAIL);
        } else
            mFufingerSDK.LedLight(0);
    }

    private void GetEnrollListBtnClick() {
        String msg = "";
        for (int i = 0; i < zaclient.getfingercharsSize(); i++) {
            msg += i + ",";
        }
        showmsg("fingerlist:" + msg);
    }

    private void emptyBtnClick() {
        zaclient.Clearfingerchars();
        showmsg(getString(R.string.Emptyok));
    }

//    private void LFDBtnClick() {
//        showmsg(getString(R.string.start));
//        FPSPLIT_INFO[] info = new FPSPLIT_INFO[10];
//        int SPLIT_IMG_WIDTH = 256;
//        int SPLIT_IMG_HEIGHT = 360;
//        int IMG_WIDTH = 1600;
//        int IMG_HEIGHT = 1500;
//        int mcount = 0;
//        for (int i = 0; i < 10; ++i) {
//            info[i] = new FPSPLIT_INFO();
//            info[i].pOutBuf = new byte[SPLIT_IMG_WIDTH * SPLIT_IMG_HEIGHT];
//        }
//        mFufingerSDK.LIVESCAN_Beep(1);
//        long starttime = System.currentTimeMillis();
//        while (isonline && (System.currentTimeMillis() - starttime) < 10000) {
//            byte[] rawdata = new byte[IMG_WIDTH * IMG_HEIGHT];
//            int ret = mFufingerSDK.captureFinger(100, rawdata, null);
//            if (ret == 1) {
//                showFingerImg(rawdata, IMG_WIDTH, IMG_HEIGHT);
//            } else {
//                continue;
//            }
//
//            int fingerCount = FingerAlgo.DoSplit(rawdata, IMG_WIDTH, IMG_HEIGHT, SPLIT_IMG_WIDTH, SPLIT_IMG_HEIGHT, info);
//            if (fingerCount > 0) {
//                mcount++;
//                if (mcount < 5) continue;
//                mcount = 0;
//                boolean isQualityGood = true;
//                for (int i = 0; i < fingerCount; ++i) {
//                    ret = FingerAlgo.MOSAIC_FingerQuality(info[i].pOutBuf, SPLIT_IMG_WIDTH, SPLIT_IMG_HEIGHT);
//                    if (ret < FingerQuality) {
//                        isQualityGood = false;
//                        break;
//                    }
//                }
//                if (isQualityGood) {
//                    //判断真假手指
//                    boolean isfake = false;
//                    for (int i = 0; i < fingerCount; ++i) {
//                        if (!(jniFinger.JniGetImageAreaHary(info[i].pOutBuf, SPLIT_IMG_WIDTH, SPLIT_IMG_HEIGHT) == 1
//                                && (jniFinger.JniGetImagePore(info[i].pOutBuf, SPLIT_IMG_WIDTH, SPLIT_IMG_HEIGHT) == 1
//                                || jniFinger.JniGetImageUniform(info[i].pOutBuf, SPLIT_IMG_WIDTH, SPLIT_IMG_HEIGHT) == 1))) {
//                            isfake = true;
//                            break;
//                        }
//                    }
//                    if (isfake) {
//                        showmsg(getString(R.string.fakefp));
//                    } else {
//                        showmsg(getString(R.string.notfakefp));
//                    }
//                    mFufingerSDK.LIVESCAN_Beep(2);
//                    return;
//                }
//            }
//
//        }
//        showmsg(getString(R.string.timeout));
//        mSoundPlayer.playShortResource(16);
//    }

    int wsqw = 0;
    int wsqh = 0;
    private void showFingerImg(byte[] fingerImg, int w, int h) {
        if (fingerImg == null) {
            runOnUiThread(() -> imageView.setImageBitmap(null));
            return;
        }

        byte[] bmpBytes = create8bitGrayscaleBMP(fingerImg, w, h);

        Bitmap bmp = BitmapFactory.decodeByteArray(bmpBytes, 0, bmpBytes.length);
        runOnUiThread(() -> {
            imageView.setImageBitmap(bmp);
            wsqw = w;
            wsqh = h;
            rawdatas = fingerImg;
            bmpdata = bmp;
        });
    }

    /**
     * 构造 8-bit 灰度 BMP 字节流
     */
    private byte[] create8bitGrayscaleBMP(byte[] grayPixels, int w, int h) {
        int paletteSize = 256 * 4;           // 256 色 * (B,G,R,Reserved)
        int dataOffset  = 14 + 40 + paletteSize;
        int fileSize    = dataOffset + w * h;

        ByteBuffer buffer = ByteBuffer.allocate(fileSize);
        buffer.order(ByteOrder.LITTLE_ENDIAN);

        // BITMAPFILEHEADER (14 bytes)
        buffer.putShort((short) 0x4D42);          // 'BM'
        buffer.putInt(fileSize);                  // file size
        buffer.putInt(0);                         // reserved
        buffer.putInt(dataOffset);                // offset to pixel data

        // BITMAPINFOHEADER (40 bytes)
        buffer.putInt(40);                        // header size
        buffer.putInt(w);
        buffer.putInt(h);
        buffer.putShort((short) 1);               // planes
        buffer.putShort((short) 8);               // 8 bits per pixel
        buffer.putInt(0);                         // compression = BI_RGB
        buffer.putInt(w * h);                     // image size
        buffer.putInt(0);                         // X pixels per meter
        buffer.putInt(0);                         // Y pixels per meter
        buffer.putInt(256);                       // colors used
        buffer.putInt(0);                         // important colors

        // 灰度调色板：B=G=R=gray
        for (int i = 0; i < 256; i++) {
            buffer.put((byte) i);   // B
            buffer.put((byte) i);   // G
            buffer.put((byte) i);   // R
            buffer.put((byte) 0);   // reserved
        }

        // 像素数据（BMP 自下而上存储）
        int padding = (4 - (w % 4)) & 3;
        for (int y = h - 1; y >= 0; y--) {
            buffer.put(grayPixels, y * w, w);
            // 行填充
            for (int p = 0; p < padding; p++) buffer.put((byte) 0);
        }

        return buffer.array();
    }
    private void showFingerImg1(byte[] fingerImg, int w, int h) {
        if (fingerImg == null) {
            runOnUiThread(new Runnable() {
                public void run() {
                    imageView.setImageBitmap(null);
                }
            });
            return;
        }

        int[] pixels = new int[w * h];
        Bitmap result = Bitmap.createBitmap(w, h, Bitmap.Config.RGB_565);
        int alpha = 0xFF << 24;
        for (int i = 0; i < h; i++) {
            for (int j = 0; j < w; j++) {
                int grey = fingerImg[w * i + j] & 0XFF;

                grey = alpha | (grey << 16) | (grey << 8) | grey;

                pixels[w * i + j] = grey;

            }
        }

        runOnUiThread(new Runnable() {
            public void run() {
                 result.setPixels(pixels, 0, w, 0, 0, w, h);
                 imageView.setImageBitmap(result);

                bmpdata = result;//result;

            }
        });

    }
    private void savebmpBtnClick() {
        byte[] data = new byte[wsqw*wsqh];
        byte[] wsqdata = new byte[wsqw*wsqh];
        if (rawdatas == null && bmpdata == null) return;

        File dir = new File(Environment.getExternalStorageDirectory(), "finger");
        if (!dir.exists()) dir.mkdirs();

        long ts = System.currentTimeMillis();
        /* 1. 写 .raw */
        if (rawdatas != null) {
            File rawFile = new File(dir, "finger" + ts + ".raw");
            FileOutputStream fos = null;
            try {
                fos = new FileOutputStream(rawFile);
                fos.write(rawdatas);
                fos.flush();
            } catch (IOException e) {
                e.printStackTrace();
            } finally {
                if (fos != null) try { fos.close(); } catch (IOException ignore) {}
            }
            data = rawdatas;

            Log.e("log","log F_CompressToWSQImage 1111111111111"  );
            int wsqsize = zaclient.F_CompressToWSQImage(usbdevice,data,wsqw,wsqh,wsqdata);
            Log.e("log","log F_CompressToWSQImage wsqsize = " +wsqsize);

            int[] ww = new int[1];
            int[] hh = new int[1];
            wsqsize = zaclient.F_UnCompressFromWSQImage(usbdevice,wsqdata,wsqsize,wsqdata,ww,hh);
            Log.e("log","log F_UnCompressFromWSQImage ww = " +ww[0]  +"  hh = "+hh[0]);

            File rawFile1 = new File(dir, "finger.wsq");
            FileOutputStream fosw = null;
            try {
                fosw = new FileOutputStream(rawFile1);
                fosw.write(wsqdata);
                fosw.flush();
            } catch (IOException e) {
                e.printStackTrace();
            } finally {
                if (fosw != null) try { fosw.close(); } catch (IOException ignore) {}
            }
        }
        /* 2. 写 .bmp */
        if (bmpdata != null) {
            File bmpFile = new File(dir, "finger" + ts + ".bmp");
            writeBmp(bmpdata, bmpFile);
        }

        /* 3. 写 .png */
        if (bmpdata != null) {
            File pngFile = new File(dir, "finger" + ts + ".png");
            FileOutputStream fos = null;
            try {
                fos = new FileOutputStream(pngFile);
                bmpdata.compress(Bitmap.CompressFormat.PNG, 100, fos);
                fos.flush();
            } catch (IOException e) {
                e.printStackTrace();
            } finally {
                if (fos != null) try { fos.close(); } catch (IOException ignore) {}
            }
        }

        /* 4. 写 .jpg */
        if (bmpdata != null) {
            File jpgFile = new File(dir, "finger" + ts + ".jpg");
            FileOutputStream fos = null;
            try {
                fos = new FileOutputStream(jpgFile);
                bmpdata.compress(Bitmap.CompressFormat.JPEG, 95, fos);
                fos.flush();
            } catch (IOException e) {
                e.printStackTrace();
            } finally {
                if (fos != null) try { fos.close(); } catch (IOException ignore) {}
            }
        }

        Toast.makeText(MainActivity.this, "已保存到 " + dir.getAbsolutePath(), Toast.LENGTH_SHORT).show();
//        byte[] data = new byte[size1];

//        try {
//            File myFile = new File(dir, "finger.wsq");
//            data =   fileToBytes(myFile);
//            byte[] raw =new byte[wsqw*wsqh] ;
//            int[] w1 = new int[1];
//            int[] h1 = new int[1];
//
//            zaclient.F_UnCompressFromWSQImage(usbdevice,data,size1,raw,w1,h1);
//          //  showFingerImg(data,w1[0],h1[0]);
//        } catch (IOException e) {
//            throw new RuntimeException(e);
//        }

//        if (bmpdata == null) return;
//        String path = Environment.getExternalStorageDirectory().getAbsolutePath() + "/finger" + System.currentTimeMillis() + ".png";
//        File file = new File(path);
//        FileOutputStream fileOutputStream = null;
//        try {
//            fileOutputStream = new FileOutputStream(file);
//            bmpdata.compress(Bitmap.CompressFormat.PNG, 100, ((OutputStream) fileOutputStream));
//            fileOutputStream.close();
//            Toast.makeText(MainActivity.this, path, Toast.LENGTH_SHORT).show();
//        } catch (IOException e) {
//            e.printStackTrace();
//        }
    }
    public static byte[] fileToBytes(File file) throws IOException {
        try (FileInputStream fis = new FileInputStream(file);
             ByteArrayOutputStream bos = new ByteArrayOutputStream()) {
            byte[] buf = new byte[4096];
            int n;
            while ((n = fis.read(buf)) != -1) bos.write(buf, 0, n);
            return bos.toByteArray();
        }
    }
    private void showmsg(String msg) {
        runOnUiThread(new Runnable() {
            public void run() {
                tvmsg.setText(msg);
            }
        });
    }
    /* ---------- 把 Bitmap 写成 24-bit BMP ---------- */
    private static void writeBmp(Bitmap bmp, File file) {
        if (bmp == null) return;
        int w = bmp.getWidth();
        int h = bmp.getHeight();
        int[] pixels = new int[w * h];
        bmp.getPixels(pixels, 0, w, 0, 0, w, h);

        // 每行字节数必须是 4 的倍数
        int rowBytes = w * 3;
        int pad = (4 - (rowBytes % 4)) % 4;
        int fileSize = 54 + (rowBytes + pad) * h;

        try (FileOutputStream fos = new FileOutputStream(file)) {
            // 文件头
            writeShort(fos, 0x4D42);          // bfType
            writeInt(fos, fileSize);          // bfSize
            writeInt(fos, 0);                 // bfReserved
            writeInt(fos, 54);                // bfOffBits

            // 信息头
            writeInt(fos, 40);                // biSize
            writeInt(fos, w);                 // biWidth
            writeInt(fos, h);                 // biHeight
            writeShort(fos, 1);               // biPlanes
            writeShort(fos, 24);              // biBitCount
            writeInt(fos, 0);                 // biCompression
            writeInt(fos, (rowBytes + pad) * h); // biSizeImage
            writeInt(fos, 0x0B13);            // biXPelsPerMeter
            writeInt(fos, 0x0B13);            // biYPelsPerMeter
            writeInt(fos, 0);                 // biClrUsed
            writeInt(fos, 0);                 // biClrImportant

            // 像素数据（倒序写入）
            byte[] line = new byte[rowBytes + pad];
            for (int y = h - 1; y >= 0; y--) {
                int idx = 0;
                for (int x = 0; x < w; x++) {
                    int px = pixels[y * w + x];
                    line[idx++] = (byte) (px & 0xFF);       // B
                    line[idx++] = (byte) ((px >> 8) & 0xFF); // G
                    line[idx++] = (byte) ((px >> 16) & 0xFF); // R
                }
                // 补 0
                for (int i = 0; i < pad; i++) line[idx++] = 0;
                fos.write(line);
            }
            fos.flush();
        } catch (IOException e) {
            e.printStackTrace();
        }
    }

    private static void writeShort(FileOutputStream fos, int v) throws IOException {
        fos.write(v & 0xFF);
        fos.write((v >> 8) & 0xFF);
    }

    private static void writeInt(FileOutputStream fos, int v) throws IOException {
        fos.write(v & 0xFF);
        fos.write((v >> 8) & 0xFF);
        fos.write((v >> 16) & 0xFF);
        fos.write((v >> 24) & 0xFF);
    }
/*    private void SetButton(Button button, boolean b) {
        runOnUiThread(new Runnable() {
            public void run() {
                if (b) button.setBackgroundResource(R.drawable.rounded_button);
                else button.setBackgroundResource(R.drawable.rounded_button_click);
                button.setEnabled(b);
            }
        });
    }*/

    private void SetButton(boolean b) {
        runOnUiThread(new Runnable() {
            public void run() {
                int count = 0;
                if (isonline) count = 1;
                for (int i = count; i < btnarr.length; i++) {
                    btnarr[i].setEnabled(b);
                }
            }
        });
    }

    @Override
    protected void onStop() {
        super.onStop();
    }

    @Override
    protected void onStart() {
        super.onStart();

    }


    private void Sleep(long time) {
        try {
            Thread.sleep(time);
        } catch (InterruptedException e) {
            e.printStackTrace();
        }
    }

    private void checkPermission() {
        mPermissionList.clear();
        //判断哪些权限未授予
        for (int i = 0; i < permissions.length; i++) {
            if (ContextCompat.checkSelfPermission(this, permissions[i]) != PackageManager.PERMISSION_GRANTED) {
                mPermissionList.add(permissions[i]);
            }
        }
        /**
         * 判断是否为空
         */
        if (mPermissionList.isEmpty()) {//未授予的权限为空，表示都授予了
        } else {//请求权限方法
            String[] permissions = mPermissionList.toArray(new String[mPermissionList.size()]);//将List转为数组
            ActivityCompat.requestPermissions(MainActivity.this, permissions, PERMISSION_REQUEST);
        }
    }

    public void onRequestPermissionsResult(int requestCode, @NonNull String[] permissions, @NonNull int[] grantResults) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults);
        switch (requestCode) {
            case PERMISSION_REQUEST:
                break;
            default:
                super.onRequestPermissionsResult(requestCode, permissions, grantResults);
                break;
        }
    }


}