package com.antiris;

import android.annotation.SuppressLint;
import android.app.AlertDialog;
import android.content.Context;
import android.content.DialogInterface;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.content.res.Configuration;
import android.graphics.Bitmap;
import android.graphics.Color;
import android.graphics.Point;
import android.hardware.usb.UsbDevice;
import android.os.Build;
import android.os.Bundle;
import android.os.Environment;
import android.text.TextUtils;
import android.util.DisplayMetrics;
import android.util.TypedValue;
import android.view.Gravity;
import android.view.Menu;
import android.view.MenuItem;
import android.view.View;
import android.view.ViewGroup;
import android.view.Window;
import android.view.WindowManager;
import android.widget.Button;
import android.widget.CheckBox;
import android.widget.ImageView;
import android.widget.LinearLayout;
import android.widget.PopupMenu;
import android.widget.RadioButton;
import android.widget.SeekBar;
import android.widget.TextView;
import android.widget.Toast;

import androidx.annotation.NonNull;
import androidx.annotation.Nullable;
import androidx.annotation.RequiresApi;
import androidx.appcompat.app.AppCompatActivity;

import com.antiris.AntUSBCamera.DeviceConnectCallback;
import com.antiris.AntUSBCamera.EnrollCallback;
import com.antiris.AntUSBCamera.IdentifyCallback;
import com.antiris.AntUSBCamera.IrexCaptureCallback;
import com.antiris.AntUSBCamera.StreamType;
import com.antiris.AntUSBCamera.USBControl;
import com.antiris.AntUSBCamera.data.Iris;
import com.antiris.AntUSBCamera.iso.IrisIso;
import com.antiris.AntUSBCamera.utils.BitmapUtil;
import com.antiris.AntUSBCamera.utils.FileUtil;
import com.antiris.AntUSBCamera.utils.LanguageUtil;
import com.antiris.AntUSBCamera.utils.Log;
import com.antiris.AntUSBCamera.utils.LogCat;
import com.antiris.AntUSBCamera.utils.Permission;
import com.antiris.AntUSBCamera.utils.SoundManager;
import com.antiris.AntUSBCamera.utils.Timer;
import com.antiris.AntUSBCamera.utils.TimerCallback;
import com.antiris.AntUSBCamera.widget.CustomProgressCircle;
import com.antiris.AntUSBCamera.widget.Draw;

import org.json.JSONObject;

import java.io.BufferedReader;
import java.io.File;
import java.io.FileInputStream;
import java.io.FileWriter;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.security.NoSuchAlgorithmException;
import java.text.SimpleDateFormat;
import java.util.ArrayList;
import java.util.Date;
import java.util.List;
import java.util.concurrent.atomic.AtomicBoolean;

public class MainActivity extends AppCompatActivity implements DeviceConnectCallback,View.OnClickListener {
    private static final String TAG = "iris";
    private  boolean mInit = false;
    public View mUSBCameraImageTexView;
    static private USBControl mCameraHelper;
    private Button mEnrollBtn;
    private Button mVerifyBtn;
    private Button mIREXCaptureBtn;
    private Button mClearBtn;
    private  Button _buttonStopStream;
    private  Button mStreamTypeBtn;
    private Timer mTimerHideHintText;
    private Timer mTimerSaveAlertDlg;
    private  long _showHintTextTimeMillis;
    private TextView mHintTv;
    private SoundManager mSoundManager;
    private CheckBox mConnected;
    private CheckBox _cbViewMirror;
    private  CheckBox mCBViewFlip;
    private  CheckBox mManual;
    private  boolean _viewMirror;
    private  boolean mViewFlip;
    private boolean mRestartPreview = false;

    private  int mStreamType = StreamType.STREAM_TYPE_IRIS;
    private CustomProgressCircle _enrollProgress;
    private List<Iris> irises = new ArrayList<>();
    private SeekBar _seekBarWhitLed;
    private  TextView _tvWhiteLed;
    private  int _whitLedValue = 0;
    private  int mIrexType = 1;
    private  String mCurrentLanguage ;
    private RadioButton buttonLeftEye;
    private RadioButton buttonRightEye;
    private  RadioButton buttonTwoEyes;
    private ImageView _k7ImageLeft;
    private ImageView _k7ImageRight;
    private ImageView mMore;

    private  InputDialog enrollDialog;
    private  InputDialog irexCaptureDialog;
    private RadioButton buttonK1;
    private RadioButton buttonK2;
    private RadioButton buttonK3;
    private RadioButton buttonK7;

    private Draw draw;

    private String _templatePath ;
    private  String _irisUsersFileName = "IrisUser";
    private  int _editIrisIndex = -1;
    private  int _furthers = 0;
    private  int _closers = 0;
    private  int _noWideOpening = 0;
    private  int mScreenHeight = 0;
    private  int mScreenWidth = 0;
    private  int mProgress = 0;
    boolean needIdentify = false;
    boolean needEnroll = false;
    boolean needCapture = false;
    boolean mPowerSavingMode = false;
    static boolean mOpButtonLeft = false;
    static LogCat logcat;
    static private final int SOUND_ENROLL_SUCCESS = 1;
    static private final int SOUND_IDENTIFY_SUCCESS = 2;
    static private final int SOUND_MOVE_FURTHER = 3;
    static private final int SOUND_MOVE_CLOSER = 4;
    static private final int SOUND_OPEN_EYES_WIDER = 5;

    static private final int SOUND_BEEP = 6;
    static private final int SOUND_KEEP_DISTANCE = 7;
    static private final int SOUND_LOOK_CAMERA = 8;
    static private final int SOUND_CAPTURE_SUCCESS = 9;
    private Context mContext = null;

    protected  void onCreate(@Nullable Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        //send log to muticast address .because no adb logcat in some android device
        logcat = new LogCat();
        logcat.start(this,"224.0.2.1",8100,false,"");

        mContext = this;

        readConfig();

        if(mCurrentLanguage == null || mCurrentLanguage.isEmpty())
        {
            Log.d("current language is empty");

            mCurrentLanguage = LanguageUtil.getInstance().getCurrentLanguage(this);
        }


        Log.w("current lang : " + mCurrentLanguage);

        requestWindowFeature(Window.FEATURE_NO_TITLE);
        getWindow().addFlags(WindowManager.LayoutParams.FLAG_FULLSCREEN);

        WindowManager windowManager = getWindow().getWindowManager();
        Point point = new Point();
        windowManager.getDefaultDisplay().getRealSize(point);
        mScreenWidth = point.x;
        mScreenHeight = point.y;


        //If landscape, video width will be 2/3 of  screen width
        if (this.getResources().getConfiguration().orientation == Configuration.ORIENTATION_LANDSCAPE )
        {
                //if height is very small , use smaller textviews , buttons ...
                if(mOpButtonLeft)
                {
                    setContentView(R.layout.activity_camera_small_on_right);
                    LinearLayout midLayout = findViewById(R.id.rightLayout);
                    LinearLayout.LayoutParams midLayoutParams = (LinearLayout.LayoutParams) midLayout.getLayoutParams();
                    midLayoutParams.width = mScreenWidth*1/3;
                }
                else
                {
                    setContentView(R.layout.activity_camera_small);

                    LinearLayout midLayout = findViewById(R.id.midLayout);
                    LinearLayout.LayoutParams midLayoutParams = (LinearLayout.LayoutParams) midLayout.getLayoutParams();
                    midLayoutParams.width = mScreenWidth*2/3;
                }

            DisplayMetrics dm = new DisplayMetrics();
            getWindow().getWindowManager().getDefaultDisplay().getMetrics(dm);
            Log.w( "dpi " +dm.densityDpi  + " density " + dm.density);


            if(dm.density > (float) 2.5)
            {
                scaleChildSize(findViewById(R.id.rightLayout),(float)2.5/dm.density);

            }

        }
        else
        {
            setContentView(R.layout.activity_camera);
        }


        //requre necessary permisson
        if(!Permission.isPermissionGranted(this))
        {
            Permission.checkPermission(this);
            return;
        }


        init();


    }

    @Override
    public void onRequestPermissionsResult(
            int requestCode, @NonNull String[] permissions, @NonNull int[] grantResults) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults);
        if (requestCode == Permission.REQUEST_CODE) {
            for (int grantResult : grantResults) {
                if (grantResult != PackageManager.PERMISSION_GRANTED) {
                    Log.e(TAG,"permisson failed");
                    this.finish();
                    return;
                }
                else
                {
                    if(Permission.isPermissionGranted(this))
                    {
                        init();
                    }
                }
            }
        }
    }


    @Override protected void onActivityResult(int requestCode, int resultCode, Intent data) {
        super.onActivityResult(requestCode, resultCode, data);
        if (requestCode == Permission.REQUEST_CODE) {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R ) {// android 11
                if (Environment.isExternalStorageManager()) {
                    if(Permission.isPermissionGranted(this))
                    {
                        init();

                    }
                }
                else
                {
                    Log.e(TAG,"permisson failed");
                    this.finish();
                    return;
                }
            }

        }

    }

    private  void scaleChildSize(LinearLayout ll,float scale)
    {
        int childCount = ll.getChildCount();
        for (int i = 0; i < childCount;i++)
        {
            View v = ll.getChildAt(i);
            if(null == v)
            {
                continue;
            }
            //Log.d("view tag : " + v.getTag().toString());
            if(v instanceof Button)
            {
                Button btn = (Button) v;
               // Log.w("Button text size : " + btn.getTextSize());
                btn.setTextSize(TypedValue.COMPLEX_UNIT_PX,btn.getTextSize()*scale);

            }
            else if(v instanceof TextView)
            {
                TextView tv = (TextView) v;
                // Log.d("TextView text size : " + tv.getTextSize());

                tv.setTextSize(TypedValue.COMPLEX_UNIT_PX,tv.getTextSize()*scale);
            }
            else if(v instanceof ImageView)
            {
                ImageView iv = (ImageView) v;

                int id = iv.getId();
                if(id != R.id.k7imageLeft && id !=  R.id.k7imageRight)
                {
                   ViewGroup.LayoutParams lp = iv.getLayoutParams();

                    Log.w("ImageView   iv.getLayoutParams().width : " +  lp.width);

                    lp.width = (int)((float)lp.width*scale);
                    lp.height = (int)((float)lp.height*scale);

                   iv.setLayoutParams(lp);
                }

            }
            else if(v instanceof LinearLayout)
            {
                LinearLayout llv = (LinearLayout)v;
                scaleChildSize(llv,scale);

                ViewGroup.LayoutParams llvPara = llv.getLayoutParams();

                //llvPara.height= (int) ((float)llvPara.height *scale);

            }
        }

    }

    public  void init()
    {
        if(mInit)
        {
            return;
        }

        initView();

        try {
            initData();
        } catch (NoSuchAlgorithmException e) {
            e.printStackTrace();
        } catch (IOException e) {
            e.printStackTrace();
        }

        initListener();


        mInit = true;
    }


    private  void saveConfig()
    {
        try {

            JSONObject jsonObject = new JSONObject();
            jsonObject.put("enroll_type", mIrexType);
            jsonObject.put("language", mCurrentLanguage);

            File file = new File(getExternalFilesDir(null).getAbsolutePath(), "config");

            FileWriter writer = new FileWriter(file);
            writer.write(jsonObject.toString());
            writer.flush();
            writer.close();

        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    private   void readConfig()
    {
        try {
            File file = new File(getExternalFilesDir(null).getAbsolutePath(), "config");
            if (!file.exists())
            {
                Log.d("read config error");
                return ;
            }

            FileInputStream inputStream = new FileInputStream(file);
            BufferedReader reader = new BufferedReader(new InputStreamReader(inputStream));
            StringBuilder jsonString = new StringBuilder();
            String line;

            while ((line = reader.readLine()) != null) {
                jsonString.append(line);
            }

            reader.close();
            inputStream.close();

            JSONObject jsonObject = new JSONObject(jsonString.toString());

            mIrexType = jsonObject.getInt("enroll_type");
            mCurrentLanguage = jsonObject.getString("language");

        } catch (Exception e) {
            e.printStackTrace();
        }
    }


    @SuppressLint("HandlerLeak")
    public void initView() {
        mUSBCameraImageTexView=findViewById(R.id.camera_view);
        _viewMirror = false;

        mUSBCameraImageTexView.setScaleX(-1);//no mirror

        _enrollProgress = findViewById(R.id.enroll_progress);
        draw=findViewById(R.id.oval);
      //  _etUserName= findViewById(R.id.user_name);
       buttonLeftEye = findViewById(R.id.left_eye);
       buttonRightEye = findViewById(R.id.right_eye);
        buttonTwoEyes = findViewById(R.id.two_eyes);
        _k7ImageLeft=findViewById(R.id.k7imageLeft);
        _k7ImageRight=findViewById(R.id.k7imageRight);
        mConnected=findViewById(R.id.ck_camera);
        mManual = findViewById(R.id.ck_manual);
        mEnrollBtn = findViewById(R.id.enrollBtn);
        mVerifyBtn = findViewById(R.id.verifyBtn);
        mHintTv = findViewById(R.id.hintTv);
        mIREXCaptureBtn = findViewById(R.id.irex_capture);
        mClearBtn = findViewById(R.id.clear);
       /* buttonK1 = findViewById(R.id.k1);
        buttonK2 = findViewById(R.id.k2);
        buttonK3 = findViewById(R.id.k3);
        buttonK7 = findViewById(R.id.k7);*/
        mMore = findViewById(R.id.more);
        mStreamTypeBtn = findViewById(R.id.stream_type);

        _buttonStopStream = findViewById(R.id.stopstream);
        _seekBarWhitLed = findViewById(R.id.seekbar_white_led);
        _tvWhiteLed = findViewById(R.id.hint_white_led);
        _cbViewMirror= findViewById(R.id.view_mirror);
        mCBViewFlip = findViewById(R.id.view_flip);
        _tvWhiteLed.setVisibility(View.GONE);
        _seekBarWhitLed.setVisibility(View.GONE);

       // Log.w("system lang : " + LanguageUtil.getInstance().getCurrentLanguage(this));

        LanguageUtil.getInstance().changeLanguage(this,mCurrentLanguage);
        updateTexts();
    }

    public void initData() throws IOException, NoSuchAlgorithmException {

        _templatePath = Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_PICTURES).toString() + "/Antiris/template/";

        FileUtil.makeDir(_templatePath);

        mCameraHelper = USBControl.getInstance();

        //init usb camera monitor and connection
        if(!mPowerSavingMode)
        {
            mCameraHelper.init(this,  mUSBCameraImageTexView, this,true);
        }
        else
        {
            _buttonStopStream.setText(R.string.start_stream);
        }

        //load template form files
        loadTemple();

        //init enroll and identity success sound
        updateSounds();

        showEnrollProgress(false);
        _enrollProgress.mirror(_viewMirror);
    }

    //show different control by device model
    private void updateLayout(){
        if (mCameraHelper.isMonocular()){
            buttonLeftEye.setVisibility(View.VISIBLE);
            buttonRightEye.setVisibility(View.VISIBLE);
            buttonLeftEye.setChecked(true);
            buttonTwoEyes.setVisibility(View.GONE);
        }else {
            _k7ImageRight.setVisibility(View.VISIBLE);
            buttonLeftEye.setVisibility(View.VISIBLE);
            buttonRightEye.setVisibility(View.VISIBLE);
            buttonTwoEyes.setVisibility(View.VISIBLE);
            buttonTwoEyes.setChecked(true);


        }

        if(mCameraHelper.antGetStreamType(StreamType.STREAM_TYPE_FACE))
        {
            mStreamTypeBtn.setVisibility(View.VISIBLE);
        }
        else
        {
            mStreamTypeBtn.setVisibility(View.GONE);
        }

        if(mCameraHelper.needWhiteLed())
        {
            _tvWhiteLed.setVisibility(View.VISIBLE);
            _seekBarWhitLed.setVisibility(View.VISIBLE);
        }
        else
        {
            _tvWhiteLed.setVisibility(View.GONE);
            _seekBarWhitLed.setVisibility(View.GONE);
        }
    }

    //show or hide enroll progress
    private  void showEnrollProgress(boolean show)
    {
        if(show)
        {
            _enrollProgress.setVisibility(View.VISIBLE);
        }
        else
        {
            _enrollProgress.setProgress(2,0);
            _enrollProgress.setVisibility(View.GONE);
        }
    }

    public void initListener() {
        mEnrollBtn.setOnClickListener(this);
        mVerifyBtn.setOnClickListener(this);
        mIREXCaptureBtn.setOnClickListener(this);
        mClearBtn.setOnClickListener(this);
        mMore.setOnClickListener(this);
        mStreamTypeBtn.setOnClickListener(this);
        // saveVideo.setOnClickListener(this);
        _buttonStopStream.setOnClickListener(this);
        _cbViewMirror.setOnClickListener(this);
        mCBViewFlip.setOnClickListener(this);

        _seekBarWhitLed.setOnSeekBarChangeListener(new SeekBar.OnSeekBarChangeListener() {
            @RequiresApi(api = Build.VERSION_CODES.JELLY_BEAN)
            @Override
            public void onProgressChanged(SeekBar seekBar, int progress, boolean fromUser) {
                _whitLedValue = progress;
            }

            @Override
            public void onStartTrackingTouch(SeekBar seekBar) {
            }

            @Override
            public void onStopTrackingTouch(SeekBar seekBar) {
                if (mCameraHelper != null && mCameraHelper.isCameraOpened()) {
                    mCameraHelper.setWhiteLed(_whitLedValue);
                }
            }
        });
    }

    @Override
    protected void onStart(){
        super.onStart();
        Log.w("onStart");

        //clear hint text in 2 seconds
        mTimerHideHintText = new Timer();
        mTimerHideHintText.schedule(1,1000,1000, (TimerCallback) timerID -> {
            if(0 != _showHintTextTimeMillis && System.currentTimeMillis() - _showHintTextTimeMillis > 2000)
            {
                mHintTv.setText(null);
            }
        });

    }

    @Override
    protected void onStop() {
        super.onStop();
        Log.w("onStop");
        if(null != mTimerHideHintText)
        {
            mTimerHideHintText.cancel();
            mTimerHideHintText = null;
        }

    }

    @Override
    protected void onPause() {
        super.onPause();

        Log.w("onPause");

        if(null != mCameraHelper )
        {
            if( mCameraHelper.isPreviewing())
            {
                stopStream();
            }

        }
    }

    @Override
    protected void onResume() {
        super.onResume();
        Log.w("onResume");
        if(null != mCameraHelper && !mCameraHelper.isPreviewing())
        {
            stopStream();
        }
    }

    @Override
    protected void onDestroy() {
        super.onDestroy();
        Log.w("onDestroy");
        if (mCameraHelper != null) {
            mCameraHelper.release();
        }
    }

    //update hint text
    private  void setHintText(String msg,int color)
    {
        mHintTv.setTextColor(color);
        mHintTv.setText(msg);
        _showHintTextTimeMillis = System.currentTimeMillis() ;
    }

    private void showShortMsg(String msg){
        Toast.makeText(this, msg, Toast.LENGTH_SHORT).show();
    }

    @Override
    public void onAttachDev(UsbDevice device) {
            //discover valid device , update connect status,draw solution ,update layout
            if (mCameraHelper != null)
            {
                runOnUiThread(() -> {
                    mConnected.setChecked(false);
                    mConnected.setText(getString(R.string.connecting));


                    updateLayout();
                });
            }
    }

    @Override
    public void onDettachDev(UsbDevice device) {
            //device is lost,update connect status
            runOnUiThread(() -> {
                mConnected.setChecked(false);
                mConnected.setText(getString(R.string.camera_disconnected));
            });

    }

    @SuppressLint("SetTextI18n")
    @Override
    public void onConnectDev(UsbDevice device, boolean isConnected) {
        if (!isConnected) {
            showShortMsg(getString(R.string.connect_error));
        } else {
            runOnUiThread(() -> {
            //device connect success
                mConnected.setChecked(true);
                mConnected.setText(getString(R.string.camera_connected));

                _viewMirror = _cbViewMirror.isChecked();
                mUSBCameraImageTexView.setScaleX( _viewMirror?1 : -1);
                _enrollProgress.mirror(_viewMirror);
                _whitLedValue = mCameraHelper.getWhiteLed();
                _seekBarWhitLed.setProgress(_whitLedValue);

            });

        }
    }
    //device disconnect
    @Override
    public void onDisConnectDev(UsbDevice device) {
        runOnUiThread(() -> {
            showEnrollProgress(false);


        });
    }

    //device preview
    @Override
    public void onPreview(boolean previewing) {
        runOnUiThread(() -> {

            if(previewing)
            {

                mStreamTypeBtn.setEnabled(true);
                _buttonStopStream.setText(R.string.stop_stream);
                _buttonStopStream.setEnabled(true);
                _whitLedValue = mCameraHelper.getWhiteLed();

                _seekBarWhitLed.setProgress(_whitLedValue);

                if(buttonRightEye.isChecked())
                {
                    mEnrollEye = mCameraHelper.RIGHT_EYE;
                }
                else if(buttonTwoEyes.isChecked())
                {
                    mEnrollEye = mCameraHelper.TWO_EYES;
                }
                else
                {
                    mEnrollEye = mCameraHelper.LEFT_EYE;
                }

                if(needEnroll) {

                    startEnroll(mEnrollUserName,mEnrollTemplate, mEnrollEye);
                    mEnrollBtn.setEnabled(true);
                    needEnroll = false;
                }
                else if(needIdentify)
                {
                    identify();
                    mVerifyBtn.setEnabled(true);
                    needIdentify = false;
                }
                else if(needCapture)
                {
                    startIrexCapture(mEnrollUserName,mEnrollEye);
                    mIREXCaptureBtn.setEnabled(true);
                    needCapture = false;
                }

                draw.setActionTips(mCameraHelper.getPreviewWidth()+"*"+mCameraHelper.getPreviewHeight());

                int flip = mCameraHelper.antGetFlip();

                mViewFlip = (1 == flip ? true : false);
                mCBViewFlip.setChecked(mViewFlip);
            }
            else
            {
                _buttonStopStream.setText(R.string.start_stream);
                _buttonStopStream.setEnabled(true);

                if(mRestartPreview)
                {

                        stopStream();

                    mRestartPreview = false;
                }
            }


        });
    }

    @Override
    public void onClick(View v) {
        switch (v.getId()){
            case R.id.stopstream:
                stopStream();
                break;
            case R.id.enrollBtn:
                    enroll();
                break;
            case R.id.verifyBtn:
                identify();
                break  ;
            case R.id.clear: {
                clear();
            }
                break;
            case R.id.view_mirror:
            {
                mUSBCameraImageTexView.setScaleX(_viewMirror ? -1:1);
                _viewMirror = !_viewMirror;
                _enrollProgress.mirror(_viewMirror);
            }

                break;
            case R.id.view_flip:
            {
                if(mCameraHelper.antSetFlip(!mViewFlip))
                {
                    mViewFlip = !mViewFlip;
                    mCBViewFlip.setChecked(mViewFlip);

                }
            }
            break;
            case R.id.irex_capture:
            {
               irexCapture();
            }

            break;
            case R.id.stream_type:
                switchStreamType();
                break;
            case R.id.more:
                showPopupMenu(v);
        }
    }

    private void switchStreamType()
    {
        if(  StreamType.STREAM_TYPE_FACE == mStreamType)
        {
            mStreamTypeBtn.setText("FaceCAM");

            mStreamType = StreamType.STREAM_TYPE_IRIS;
        }
        else
        {
            mStreamTypeBtn.setText("IrisCAM");

            mStreamType = StreamType.STREAM_TYPE_FACE;
        }

        if(null != mCameraHelper)
        {
            mStreamTypeBtn.setEnabled(false);

            mCameraHelper.antSetStreamType(mStreamType);

            mRestartPreview = true;
        }


    }

    private  void updateSounds()
    {
        mSoundManager =new SoundManager(this);
        mSoundManager.put(SOUND_ENROLL_SUCCESS,R.raw.enroll_finish);
        mSoundManager.put(SOUND_IDENTIFY_SUCCESS,R.raw.beep);
        mSoundManager.put(SOUND_MOVE_FURTHER,R.raw.move_further);
        mSoundManager.put(SOUND_MOVE_CLOSER,R.raw.move_closer);
        mSoundManager.put(SOUND_OPEN_EYES_WIDER,R.raw.open_eyes_wider);
        mSoundManager.put(SOUND_BEEP,R.raw.beep);
        mSoundManager.put(SOUND_LOOK_CAMERA,R.raw.look_camera);
        mSoundManager.put(SOUND_KEEP_DISTANCE,R.raw.keep_distance);
        mSoundManager.put(SOUND_CAPTURE_SUCCESS,R.raw.capture_finish);
        buttonLeftEye.setChecked(true);
        buttonRightEye.setChecked(false);
        buttonTwoEyes.setChecked(false);
    }

    private void updateTexts()
    {
        if(null != mCameraHelper)
        {
            mVerifyBtn.setText( getString(mCameraHelper.isIdentify()?R.string.stop_verify:R.string.text_verify));
            mEnrollBtn.setText( getString(mCameraHelper.isEnrolling()?R.string.stop_enroll:R.string.text_enroll));
            mIREXCaptureBtn.setText( getString(mCameraHelper.isIdentify()?R.string.start_capture:R.string.irex_capture));
            mConnected.setText( getString(mCameraHelper.isCameraOpened()?R.string.camera_connected:R.string.camera_disconnected));
            _buttonStopStream.setText( getString(mCameraHelper.isPreviewing()?R.string.stop_stream:R.string.start_stream));

        }
        else
        {
            mVerifyBtn.setText( getString(R.string.text_verify));
            mEnrollBtn.setText( getString(R.string.text_enroll));
            mIREXCaptureBtn.setText( getString(R.string.irex_capture));
            mConnected.setText( getString(R.string.camera_disconnected));
            _buttonStopStream.setText( getString(R.string.start_stream));
        }

        mManual.setText( getString(R.string.manual));

        buttonTwoEyes.setText( getString(R.string.two_eyes));
        buttonRightEye.setText( getString(R.string.right_eye));
        buttonLeftEye.setText( getString(R.string.left_eye));
        mClearBtn.setText( getString(R.string.text_cancel));


        _tvWhiteLed.setText( getString(R.string.white_LED));
        _cbViewMirror.setText( getString(R.string.mirror));
        mCBViewFlip.setText( getString(R.string.flip));

    }

    private  void changeLanguage(String language)
    {
        if( false == mCurrentLanguage.equals(language))
        {
                       /* LanguageUtil.getInstance().setLanguage(this, "language);

                        mChangingLanguage = true;

                        Intent intent = new Intent(this, MainActivity.class);
                        intent.setFlags(Intent.FLAG_ACTIVITY_CLEAR_TOP);
                        startActivity(intent);
                        finish();*/

            LanguageUtil.getInstance().changeLanguage(this, language);
            mCurrentLanguage = language;

            saveConfig();


            updateTexts();

            updateSounds();

        }

    }

    private void showPopupMenu(View view) {
        PopupMenu popupMenu = new PopupMenu(this, view);
        popupMenu.getMenuInflater().inflate(R.menu.more_context, popupMenu.getMenu());

        int maxFixedItemID = 8;

        int itemID = R.id.k1;
        if (2 == mIrexType)
        {
            itemID = R.id.k2;
        }
        else if(3 == mIrexType)
        {
            itemID = R.id.k3;
        }
        else if(7 == mIrexType)
        {
            itemID = R.id.k7;
        }


        MenuItem itemChecked = popupMenu.getMenu().findItem(itemID);
        if(null != itemChecked)
        {
            itemChecked.setChecked(true);
        }

        itemID = R.id.en;

        Log.w("current lang : " + mCurrentLanguage);
       // if(LanguageUtil.getInstance().getCurrentLanguage(this).equals("zh"))
        if(mCurrentLanguage.equals("zh"))
        {
            itemID = R.id.zh;
        }

        itemChecked = popupMenu.getMenu().findItem(itemID);

        if(null != itemChecked)
        {
            itemChecked.setChecked(true);
        }

        Menu menuMore = popupMenu.getMenu();

        List<String> listDevID = mCameraHelper.antGetDeviceList();

        for (int i=0;i<listDevID.size();i++)
        {
            String devID = listDevID.get(i);
            String title = "IrisScanner " + devID;
            MenuItem item = menuMore.add(3, Menu.FIRST + maxFixedItemID + Integer.valueOf(devID).intValue(), 0, title);

            item.setCheckable(true);
            if(Integer.valueOf(devID).intValue() == mCameraHelper.antGetReqDeviceID())
            {
                item.setChecked(true);
            }
        }

        popupMenu.setOnMenuItemClickListener(item -> {

            final  int itemIDClick = item.getItemId();

            switch (itemIDClick)
            {
                case R.id.save_pic:
                    capture();
                    break;
                case R.id.k1:mIrexType = 1;saveConfig();break;
                case R.id.k2:mIrexType = 2;saveConfig();break;
                case R.id.k3:mIrexType = 3;saveConfig();break;
                case R.id.k7:mIrexType = 7;saveConfig();break;
                case R.id.op_button_pos:
                {
                    mOpButtonLeft = !mOpButtonLeft;
                    recreate();
                }
                    break;
                case R.id.en:
                {
                    changeLanguage("en");
                }
                break;
                case R.id.zh:
                {
                    changeLanguage("zh");



                }
                break;
                default:
                {
                    int devID = itemIDClick - Menu.FIRST - maxFixedItemID;
                    if(0 < devID && mCameraHelper.antGetReqDeviceID() != devID)
                    {
                        mCameraHelper.antOpenDeviceByID(devID);
                    }
                }
                    break;
            }
            return false;
        });
        popupMenu.setOnDismissListener(new PopupMenu.OnDismissListener() {
            @Override
            public void onDismiss(PopupMenu menu) {
                // Toast.makeText(getApplicationContext(), "close PopupMenu", Toast.LENGTH_SHORT).show();
            }
        });

        popupMenu.show();
    }

    //clear template file
    private void clear()  {

        if (null != mCameraHelper) {
            mCameraHelper.antStopEnroll(false);
            mCameraHelper.clearTemplate();
        }

        irises.clear();

        FileUtil.deleteDirWithFile(new File(_templatePath));
    }

    //save back image to file
    private void capture() {

        String _extStorageDirectory = Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_PICTURES).toString();
        String filePath = _extStorageDirectory +"/Antiris/SPicture/";
        SimpleDateFormat format = new SimpleDateFormat("yyyy-MM-dd-HHmmss");

        FileUtil.makeDir(filePath);

        String timestamp = format.format(new Date());
        String outName =  timestamp +".raw";

        byte[] bgImage = mCameraHelper.getBackgroundBytes(false);
        if(null != bgImage)
        {
            //save bg raw data
            FileUtil.saveFile(filePath,bgImage,outName);

            String k7PngOutName =  timestamp +".png";

            mCameraHelper.saveGrayToPng(bgImage,filePath,k7PngOutName,mCameraHelper.getPreviewWidth(),mCameraHelper.getPreviewHeight());
            //save bg bmp data

            String bgBmpOutName =  timestamp +".bmp";
           // mCameraHelper.saveGrayToBitmap(bgImage,filePath,bgBmpOutName,mCameraHelper.getPreviewWidth(),mCameraHelper.getPreviewHeight(),false);

            String bgJp2OutName =  timestamp +".jp2";
           // mCameraHelper.saveGrayToJp2(bgImage,filePath,bgJp2OutName,mCameraHelper.getPreviewWidth(),mCameraHelper.getPreviewHeight());
        }

    }

    private String mEnrollUserName;
    private byte[] mEnrollTemplate;
    private int mEnrollEye;
    //start or stop camera stream
    private  void stopStream()
    {
        _buttonStopStream.setEnabled(false);
        showEnrollProgress(false);

        if(null != mCameraHelper)
        {
            if(mCameraHelper.isCameraOpened())
            {
                _whitLedValue = 0;
                mCameraHelper.setWhiteLed(_whitLedValue);
                _seekBarWhitLed.setProgress(_whitLedValue);

                if(mCameraHelper.isPreviewing())
                {
                    mCameraHelper.antStopIdentify();
                    mCameraHelper.antStopEnroll(false);
                }

                mCameraHelper.stopStream();

                onUpdateIdentifyButtonStatus();
                onUpdateEnrollButtonStatus();

                if(mPowerSavingMode)
                {
                    mCameraHelper.release();
                }
            }
            else
            {
                if(mPowerSavingMode)
                {
                    mCameraHelper.init(this,  mUSBCameraImageTexView, this,true);
                }


            }

        }

    }

    private void saveKImage(int eyeIndex,String userName,boolean enroll,boolean saveJp2,boolean saveIso)
    {
        String _extStorageDirectory = Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_PICTURES).toString();
        String filePath = _extStorageDirectory +"/antiris/KImage/";
        SimpleDateFormat format = new SimpleDateFormat("yyyy-MM-dd-HHmmss");

        FileUtil.makeDir(filePath);

        String operType;

        if (enroll)
        {
            operType = "templ_";
        }
        else
        {
            operType = "irex_";
        }

        String eyeIndexString ;
        if(0 == eyeIndex)
        {
            eyeIndexString = "left";
        }
        else
        {
            eyeIndexString = "right";
        }

        byte[] k7Image = mCameraHelper.getK7Image(eyeIndex);
        if(null != k7Image)
        {
            final  int k7ImageWidth = mCameraHelper.getK7ImgWidth(eyeIndex);
            final int k7ImageHeight = mCameraHelper.getK7ImgHeight(eyeIndex);

            String resolution = k7ImageWidth + "X" + k7ImageHeight ;

            String fileNamePrefix = userName + "_" + operType +   eyeIndexString  +"_";

            String imageType = "K" + mIrexType + "_";

            File dir = new File(filePath);
            if (dir != null && dir.exists() && dir.isDirectory())
            {
                for (File file : dir.listFiles())
                {
                    if (file.isFile() && file.getName().length() >= fileNamePrefix.length())
                    {
                        String fileNameHeader = file.getName().substring(0,fileNamePrefix.length());

                        //Log.e(Log.TAG,"templatefileNameHeader:"+ templatefileNameHeader + " file.getName():" + file.getName() + " fileNameHeader:" +fileNameHeader);

                        if(fileNameHeader.equals(fileNamePrefix))
                        {
                            //Log.e(Log.TAG,"delete old file " +  file.getName());

                            file.delete();
                        }
                    }


                }
            }

            String k7PngOutName = fileNamePrefix +  imageType + resolution +".png";


            mCameraHelper.saveGrayToPng(k7Image,filePath,k7PngOutName,k7ImageWidth,k7ImageHeight);

            if(saveJp2)
            {
                    if(saveIso)
                    {
                        byte[] jp2Data = new byte[k7ImageWidth * k7ImageHeight];
                        int[] jp2Size = new int[1];

                        mCameraHelper.antSetGlobalCompressionSize(1024*10);

                        mCameraHelper.grayToJp2(k7Image,
                                k7ImageWidth,
                                k7ImageHeight,
                                jp2Data,jp2Size,
                                mIrexType,
                                mCameraHelper.ANT_COMPRESSION_MODE_LOSSY);

                        if ( jp2Size[0] > 0)
                        {
                            byte[] finalJp2Data = new byte[jp2Size[0]];

                            System.arraycopy(jp2Data,0,finalJp2Data,0,jp2Size[0]);

                            byte[] finalData = IrisIso.generate(0,finalJp2Data,
                                    k7ImageWidth,
                                    k7ImageHeight,
                                    mCameraHelper.getCameraSerialNumber().getBytes(StandardCharsets.UTF_8),
                                    mCameraHelper.getk7Quality(eyeIndex),
                                    mIrexType,
                                    mCameraHelper.ANT_IMAGE_FORMAT_JPEG2000,
                                    mCameraHelper.ANT_COMPRESSION_MODE_LOSSY,
                                    8);
                            String isoOutName = fileNamePrefix + imageType + resolution + ".iso";

                            //save iso file
                            FileUtil.saveFile(filePath,finalData,isoOutName);

                        }
                    }



            }

        }
    }
    private  int mEyeIndexEnrolled;

    //start or stop enroll
    private  void startEnroll(String userName,byte[] temple,int enrollEye)  {
        final  boolean manual = mManual.isChecked();
        mEyeIndexEnrolled = -1;
        if(!mCameraHelper.isPreviewing())
        {
            stopStream();
            needEnroll = true;
            mEnrollUserName = userName;
            mEnrollTemplate = temple;
            mEnrollEye = enrollEye;
            mEnrollBtn.setEnabled(false);
            return;
        }

        _enrollProgress.setProgress(2,0);
        showEnrollProgress(true);

        _k7ImageLeft.setImageBitmap(null);
        _k7ImageRight.setImageBitmap(null);

        mCameraHelper.antIrisSetEnrollIREXType(mIrexType);

           mCameraHelper.antIrisEnroll( userName,temple,enrollEye,100*1000,new EnrollCallback(){
            @Override
            public void onSuccess( int eyeIndex,String userName,byte[] temp) {

                if(mCameraHelper.isMonocular()
                || 0 == eyeIndex
                || (!mCameraHelper.isMonocular() && buttonRightEye.isChecked() ) )
                {
                    mSoundManager.play(SOUND_ENROLL_SUCCESS,true);

                    byte[] listData = new byte[temp.length];
                    System.arraycopy(temp,0,listData,0,temp.length);

                    byte[] byteUserName = userName.getBytes(StandardCharsets.UTF_8);
                    byte[] dstUserName = new byte[64];
                    System.arraycopy(byteUserName,0,dstUserName,0,byteUserName.length);
                    System.arraycopy(dstUserName,0,listData,0,dstUserName.length);

                    if(-1 != _editIrisIndex)
                    {
                        irises.remove(_editIrisIndex);
                    }

                    addTemple(listData,userName);

                    setHintText(getString(R.string.text_enroll_success)+ " " +  getString(R.string.user) + ":"+userName,Color.BLUE);

                    onUpdateEnrollButtonStatus();

                    String _extStorageDirectory = Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_PICTURES).toString();
                    String fileNamePrefix = userName + "_";
                    String fileDir = _extStorageDirectory +"/Antiris/SPicture/" ;
                    FileUtil.makeDir(fileDir);

                    SimpleDateFormat sdFormat = new SimpleDateFormat("yyyy-MM-dd-HHmmss");
                    String outName =  fileNamePrefix + sdFormat.format(new Date()) +".jpg";

                    File dir = new File(fileDir);
                    if (dir != null && dir.exists() && dir.isDirectory())
                    {
                        for (File file : dir.listFiles())
                        {
                            if (file.isFile() && file.getName().length() >= fileNamePrefix.length())
                            {
                                String fileNameHeader = file.getName().substring(0,fileNamePrefix.length());

                                if(fileNameHeader.equals(fileNamePrefix))
                                {
                                    file.delete();
                                }

                            }


                        }
                    }

                    mCameraHelper.antCaptureOneFaceStream(fileDir  + outName);

                }

                if(false == mCameraHelper.isDualCameras() || -1 != mEyeIndexEnrolled)
                {
                    showEnrollProgress(false);
                    mProgress = 0;
                }

                Bitmap bm  = BitmapUtil.convert8bitToBitmap(mCameraHelper.getK7Image(eyeIndex),mCameraHelper.getK7ImgWidth(eyeIndex),mCameraHelper.getK7ImgHeight(eyeIndex));

                if(0 == eyeIndex)
                {
                    _k7ImageLeft.setImageBitmap(bm);
                }
                else
                {
                    _k7ImageRight.setImageBitmap(bm);
                }

                saveKImage(eyeIndex,userName,true,false,false);

                mEyeIndexEnrolled = eyeIndex;
            }

            @Override
            public void onError( String msg) {
               // Log.i(TAG, "Enroll failed: ");
                if(!msg.isEmpty())
                {
                    String hintText = "";
                    if(mCameraHelper.ERROR_CLOSE_DISTANCE == msg)
                    {
                        mSoundManager.play(SOUND_MOVE_FURTHER,false);
                        hintText += "\n" + getString(R.string.text_further);
                        setHintText(hintText,Color.RED);
                        return;
                    }
                    else if(mCameraHelper.ERROR_FAR_DISTANCE == msg)
                    {
                        mSoundManager.play(SOUND_MOVE_CLOSER,false);
                        hintText += "\n" + getString(R.string.text_closer);
                        setHintText(hintText,Color.RED);
                        return;
                    }
                    else if(mCameraHelper.ERROR_EYE_NOT_WIDER_OPEN == msg)
                    {
                        mSoundManager.play(SOUND_OPEN_EYES_WIDER,false);
                        hintText += "\n" + getString(R.string.open_eyes_wider);
                        setHintText(hintText,Color.RED);
                        return;
                    }
                    else if(mCameraHelper.ERROR_EYE_EYE_GAZEANGLE == msg)
                    {
                        mSoundManager.play(SOUND_LOOK_CAMERA,false);
                        hintText += "\n" + getString(R.string.look_camera);
                        setHintText(hintText,Color.RED);
                        return;
                    }
                    else if(mCameraHelper.ERROR_KEEP_DISTANCE == msg)
                    {
                        mSoundManager.play(SOUND_KEEP_DISTANCE,false);
                       // hintText += "\n" + getString(R.string.keep_distance);
                       // setHintText(hintText,Color.RED);
                        return;
                    }
                    else {
                        setHintText(getString(R.string.text_enroll_fail) + ":" + msg,Color.BLUE);
                    }

                }
                else
                {
                    setHintText(msg,Color.BLUE);
                }

                onUpdateEnrollButtonStatus();

                if(!mCameraHelper.isIdentify())
                {
                    showEnrollProgress(false);
                }
                else
                {
                    _enrollProgress.setProgress(2,0);
                }

            }

            @Override
            public void onProcess(int eyeNum,int progress, int distance,int opening) {
                if (0 < progress && 100 > progress)
                {
                    mProgress = progress;
                    if (_enrollProgress.getProgress(eyeNum) != progress) {
                        _enrollProgress.setProgress(eyeNum, progress);
                    }
                }

                if (0 < distance)
                {
                    setHintText(getString(R.string.text_enroll_progress) + mProgress + "     " + distance + "cm", Color.BLUE);
                }

            }

        });

        onUpdateEnrollButtonStatus();
    }

    private void onUpdateEnrollButtonStatus()
    {
        if(mCameraHelper.isEnrolling())
        {
            buttonRightEye.setEnabled(false);
            buttonLeftEye.setEnabled(false);
            buttonTwoEyes.setEnabled(false);
            mManual.setEnabled(false);
            mEnrollBtn.setText(R.string.stop_enroll);
        }
        else
        {
            mEnrollBtn.setText(R.string.text_enroll);
            buttonRightEye.setEnabled(true);
            buttonLeftEye.setEnabled(true);
            buttonTwoEyes.setEnabled(true);
        }
    }

    private void onUpdateIREXCaptureButtonStatus()
    {
        if(mCameraHelper.isIrexCapturing())
        {
            buttonRightEye.setEnabled(false);
            buttonLeftEye.setEnabled(false);
            buttonTwoEyes.setEnabled(false);

            mManual.setEnabled(false);
            mIREXCaptureBtn.setText(R.string.stop_capture);
        }
        else
        {
            mIREXCaptureBtn.setText(R.string.irex_capture);
            buttonRightEye.setEnabled(true);
            buttonLeftEye.setEnabled(true);
            buttonTwoEyes.setEnabled(true);

            mManual.setEnabled(true);
        }
    }

    private void enroll()  {

        if (mCameraHelper != null && StreamType.STREAM_TYPE_IRIS == mStreamType){

            if(mCameraHelper.isIdentify())
            {
                mCameraHelper.antStopIdentify();
                onUpdateIdentifyButtonStatus();
                showEnrollProgress(false);
            }

            if (mCameraHelper.isIrexCapturing())
            {
                // ToastUtil.ToastNormal(getString(R.string.text_enrolling));
                mCameraHelper.antStopIrexCapture(false);
                mHintTv.setText(null);
                onUpdateIREXCaptureButtonStatus();
                _enrollProgress.setProgress(2,0);
                showEnrollProgress(false);

                needCapture = false;
            }

            if (mCameraHelper.isEnrolling()) {
                mCameraHelper.antStopEnroll(true);
                mHintTv.setText(null);
                onUpdateEnrollButtonStatus();
                showEnrollProgress(false);

                if(mPowerSavingMode && mCameraHelper.isPreviewing())
                {
                    stopStream();
                }

                needEnroll = false;

                return;
            }


            int type = mCameraHelper.LEFT_EYE;

            if(buttonRightEye.isChecked())
            {
                type = mCameraHelper.RIGHT_EYE;
            }
            else if(buttonTwoEyes.isChecked())
            {
                type = mCameraHelper.TWO_EYES;
            }

            int finalType = type;

            enrollDialog=new InputDialog(getString(R.string.iris_recogniton),this);

            enrollDialog.setPositiveListener(getString(R.string.done), (dialog, msg) -> {
                if (!TextUtils.isEmpty(msg))
                {
                    dialog.dismiss();


                        int i = 0;
                        for (; i < irises.size();i++)
                        {
                            Iris iris = irises.get(i);
                           // Log.d("msg:"+ msg + " user id:" + iris.getUser());
                            if(msg.equals(iris.getUser()))
                            {
                                _editIrisIndex = i;

                                byte[] templeBuffer = iris.getTemp();

                                int leftEyeNum =  mCameraHelper.antIrisGetEyes(templeBuffer,0);
                                int rightEyeNum = mCameraHelper.antIrisGetEyes(templeBuffer,1);

                                //Log.d("leftEyeNum:"+ leftEyeNum + " rightEyeNum:" + rightEyeNum);

                                if(mCameraHelper.LEFT_EYE == finalType)
                                {
                                    if((mCameraHelper.isMonocular() && 0 != leftEyeNum)
                                            || (false == mCameraHelper.isMonocular() && (0 != leftEyeNum || 0 != rightEyeNum)) )
                                    {
                                        String remind;
                                        if(mCameraHelper.isMonocular())
                                        {
                                            remind = getString(R.string.overwrite_left_eye);
                                        }
                                        else
                                        {
                                            if(mCameraHelper.TWO_EYES == finalType)
                                            {
                                                remind = getString(R.string.overwrite_two_eyes);
                                            }
                                            else
                                            {
                                                remind = getString(R.string.overwrite_left_eye);
                                            }
                                        }

                                        AlertDialog alertDialog2 = new AlertDialog.Builder(this)
                                                .setTitle("AntIris")
                                                .setMessage(remind)
                                                .setIcon(R.mipmap.ic_launcher)
                                                .setPositiveButton(getString(R.string.ok), new DialogInterface.OnClickListener() {//添加"Yes"按钮
                                                    @Override
                                                    public void onClick(DialogInterface dialogInterface, int i) {

                                                        if(mCameraHelper.isMonocular())
                                                        {
                                                            mCameraHelper.antIrisClearEyes(templeBuffer,mCameraHelper.LEFT_EYE);
                                                            startEnroll(msg,templeBuffer, finalType);

                                                        }
                                                        else
                                                        {
                                                            startEnroll(msg,null, finalType);

                                                        }


                                                    }
                                                })

                                                .setNegativeButton(getString(R.string.cancel), new DialogInterface.OnClickListener() {//添加取消
                                                    @Override
                                                    public void onClick(DialogInterface dialogInterface, int i) {


                                                            enroll();

                                                    }
                                                })

                                                .create();
                                        alertDialog2.show();
                                    }
                                    else
                                    {
                                        startEnroll(msg,templeBuffer, finalType);
                                    }
                                }
                                else if(mCameraHelper.RIGHT_EYE == finalType)
                                {
                                    if((mCameraHelper.isMonocular() && 0 != rightEyeNum)
                                            || (false == mCameraHelper.isMonocular() && (0 != leftEyeNum || 0 != rightEyeNum)) )
                                    {

                                        String remind;
                                        if(mCameraHelper.isMonocular())
                                        {
                                            remind =getString(R.string.overwrite_right_eye);
                                        }
                                        else
                                        {
                                            if(mCameraHelper.TWO_EYES == finalType)
                                            {
                                                remind = getString(R.string.overwrite_two_eyes);
                                            }
                                            else
                                            {
                                                remind =getString(R.string.overwrite_right_eye);
                                            }

                                        }

                                        AlertDialog alertDialog2 = new AlertDialog.Builder(this)
                                                .setTitle("AntIris")
                                                .setMessage(remind)
                                                .setIcon(R.mipmap.ic_launcher)
                                                .setPositiveButton(getString(R.string.ok), new DialogInterface.OnClickListener() {//添加"Yes"按钮
                                                    @Override
                                                    public void onClick(DialogInterface dialogInterface, int i) {


                                                        if(mCameraHelper.isMonocular())
                                                        {
                                                            mCameraHelper.antIrisClearEyes(templeBuffer,mCameraHelper.RIGHT_EYE);
                                                            startEnroll(msg,templeBuffer, finalType);

                                                        }
                                                        else
                                                        {
                                                            startEnroll(msg,null, finalType);

                                                        }


                                                    }
                                                })

                                                .setNegativeButton(getString(R.string.cancel), new DialogInterface.OnClickListener() {//添加取消
                                                    @Override
                                                    public void onClick(DialogInterface dialogInterface, int i) {

                                                        enroll();
                                                    }
                                                })

                                                .create();
                                        alertDialog2.show();
                                    }
                                    else
                                    {
                                        startEnroll(msg,templeBuffer, finalType);
                                    }
                                }
                                else
                                {
                                    if(0 != leftEyeNum || 0 != rightEyeNum )
                                    {
                                        String    remind = getString(R.string.overwrite_two_eyes);


                                        AlertDialog alertDialog2 = new AlertDialog.Builder(this)
                                                .setTitle("AntIris")
                                                .setMessage(remind)
                                                .setIcon(R.mipmap.ic_launcher)
                                                .setPositiveButton(getString(R.string.ok), new DialogInterface.OnClickListener() {//添加"Yes"按钮
                                                    @Override
                                                    public void onClick(DialogInterface dialogInterface, int i) {

                                                            startEnroll(msg,null, finalType);

                                                    }
                                                })

                                                .setNegativeButton(getString(R.string.cancel), new DialogInterface.OnClickListener() {//添加取消
                                                    @Override
                                                    public void onClick(DialogInterface dialogInterface, int i) {

                                                        enroll();
                                                    }
                                                })

                                                .create();
                                        alertDialog2.show();
                                    }
                                    else
                                    {
                                        startEnroll(msg,templeBuffer, finalType);
                                    }
                                }
                                break;
                            }




                    }

                    if(i >= irises.size())
                    {
                        _editIrisIndex = -1;
                        startEnroll(msg,null, finalType);
                    }
                }
                else
                {
                    Toast.makeText(MainActivity.this, getString(R.string.username_is_empty), Toast.LENGTH_LONG).show();
                }

            });
            enrollDialog.show();

        }
        else
        {
           /* _k7ImageRight.setVisibility(View.VISIBLE);

            mCameraHelper.k7Test();

            showK7Image(0,mCameraHelper.getK7Image(0),mCameraHelper.getK7ImgWidth(0),mCameraHelper.getK7ImgHeight(0));

            soundManager.playEnrollSuccess();*/

        }


    }


    private  void startIrexCapture(String userName,int enrollEye)  {

        final  boolean manual = mManual.isChecked();

        if(!mCameraHelper.isPreviewing())
        {
            stopStream();

            needCapture = true;
            mEnrollUserName = userName;
            mEnrollTemplate = null;
            mEnrollEye = enrollEye;
            mIREXCaptureBtn.setEnabled(false);
            return;
        }

        showEnrollProgress(true);

        mCameraHelper.antIrisIrexCapture(userName,enrollEye,mIrexType,manual,100*1000,new IrexCaptureCallback(){

            @Override
            public void onSuccess(int eyeIndex,String userName, byte[] temp){

                Bitmap bm  = BitmapUtil.convert8bitToBitmap(mCameraHelper.getK7Image(eyeIndex),mCameraHelper.getK7ImgWidth(eyeIndex),mCameraHelper.getK7ImgHeight(eyeIndex));

                if(0 == eyeIndex)
                {
                    _k7ImageLeft.setImageBitmap(bm);
                }
                else
                {
                    _k7ImageRight.setImageBitmap(bm);
                }

                onUpdateIREXCaptureButtonStatus();

                showEnrollProgress(false);
                mProgress = 0;

                AtomicBoolean autoSave = new AtomicBoolean(true);

                if(manual)
                {
                    autoSave.set(false);
                   // Log.d("eye index "+eyeIndex);
                    if( false == mCameraHelper.isBinocular() || 1 == eyeIndex)
                    {
                        mSoundManager.play(SOUND_CAPTURE_SUCCESS,true);

                        runOnUiThread(() -> {

                            AlertDialog.Builder saveAlertDlgBuilder = new AlertDialog.Builder(mContext);
                            saveAlertDlgBuilder.setMessage(mContext.getString(R.string.ask_for_save_k_image));
                            saveAlertDlgBuilder.setPositiveButton(mContext.getString(R.string.cancel), (dialog, which) -> {

                            });
                            saveAlertDlgBuilder.setNegativeButton(mContext.getString(R.string.try_again), (dialog, which) -> {
                                startIrexCapture(userName,mEnrollEye);
                            });

                            saveAlertDlgBuilder.setNeutralButton(mContext.getString(R.string.save), (dialog, which) -> {
                                if(mCameraHelper.isBinocular())
                                {
                                    saveKImage(0,userName,false,true,true);
                                }

                                saveKImage(eyeIndex,userName,false,true,true);
                            });

                            AlertDialog saveAlertDlg = saveAlertDlgBuilder.create();
                            saveAlertDlg.show();

                            saveAlertDlg.getButton(DialogInterface.BUTTON_NEGATIVE).setAllCaps(false);
                            saveAlertDlg.getButton(DialogInterface.BUTTON_POSITIVE).setAllCaps(false);
                            saveAlertDlg.getButton(DialogInterface.BUTTON_NEUTRAL).setAllCaps(false);

                        });

                    }
                }
                else
                {
                    mSoundManager.play(SOUND_CAPTURE_SUCCESS,true);
                }

                if(autoSave.get())
                {
                    saveKImage(eyeIndex,userName,false,true,true);
                }

            }

            @Override
            public void onError( String msg) {
                if(!msg.isEmpty())
                {
                    String hintText = "";
                    if(mCameraHelper.ERROR_CLOSE_DISTANCE == msg)
                    {
                        mSoundManager.play(SOUND_MOVE_FURTHER,false);
                        hintText += "\n" + getString(R.string.text_further);setHintText(hintText,Color.RED);
                        return;
                    }
                    else if(mCameraHelper.ERROR_FAR_DISTANCE == msg)
                    {
                        mSoundManager.play(SOUND_MOVE_CLOSER,false);
                        hintText += "\n" + getString(R.string.text_closer);setHintText(hintText,Color.RED);
                        return;
                    }
                    else if(mCameraHelper.ERROR_EYE_NOT_WIDER_OPEN == msg)
                    {
                        mSoundManager.play(SOUND_OPEN_EYES_WIDER,false);
                        hintText += "\n" + getString(R.string.open_eyes_wider);
                        setHintText(hintText,Color.RED);
                        return;
                    }
                    else if(mCameraHelper.ERROR_EYE_EYE_GAZEANGLE == msg)
                    {
                        mSoundManager.play(SOUND_LOOK_CAMERA,false);
                        hintText += "\n" + getString(R.string.look_camera);
                        setHintText(hintText,Color.RED);
                        return;
                    }
                    else if(mCameraHelper.ERROR_KEEP_DISTANCE == msg)
                    {
                        mSoundManager.play(SOUND_KEEP_DISTANCE,false);
                        //hintText += "\n" + getString(R.string.keep_distance);
                        //setHintText(hintText);
                        return;
                    }
                    else {
                        setHintText(getString(R.string.text_enroll_fail) + ":" + msg,Color.BLUE);
                    }
                }
                else
                {
                    setHintText(msg,Color.BLUE);
                }

                onUpdateIREXCaptureButtonStatus();

                showEnrollProgress(false);
            }

            @Override
            public void onProcess(int eyeNum,int progress, int distance,int opening) {
                if (0 < progress && 100 > progress)
                {
                    mProgress = progress;

                    if (_enrollProgress.getProgress(eyeNum) != progress && !manual) {
                        _enrollProgress.setProgress(eyeNum,progress);
                    }
                }

                if(0 < distance)
                {
                    setHintText(getString(R.string.text_enroll_progress) + mProgress + "     " + distance + "cm",Color.BLUE);
                }
            }


        });

        onUpdateIREXCaptureButtonStatus();
    }


    private void irexCapture() {


        if (mCameraHelper!=null && mCameraHelper.isCameraOpened() && StreamType.STREAM_TYPE_IRIS == mStreamType)
        {

            if(mCameraHelper.isIdentify())
            {
                mCameraHelper.antStopIdentify();
                onUpdateIdentifyButtonStatus();
                showEnrollProgress(false);
            }

            if (mCameraHelper.isIrexCapturing()) {
                // ToastUtil.ToastNormal(getString(R.string.text_enrolling));
                mCameraHelper.antStopIrexCapture(true);
                mHintTv.setText(null);
                onUpdateIREXCaptureButtonStatus();
                _enrollProgress.setProgress(2,0);
                showEnrollProgress(false);

                if(mPowerSavingMode && mCameraHelper.isPreviewing())
                {
                    stopStream();
                }

                needCapture = false;

                return;
            }


            int type = mCameraHelper.LEFT_EYE;

            if(buttonRightEye.isChecked())
            {
                type = mCameraHelper.RIGHT_EYE;
            }
            else if(buttonTwoEyes.isChecked())
            {
                type = mCameraHelper.TWO_EYES;
            }

            irexCaptureDialog=new InputDialog( getString(R.string.iris_recogniton),this);

            int finalType = type;
            irexCaptureDialog.setPositiveListener( getString(R.string.done), (dialog, msg) -> {
                if (!TextUtils.isEmpty(msg))
                {
                    dialog.dismiss();

                    final  boolean manual = mManual.isChecked();
                    final String userName = msg;
                    if(manual)
                    {
                        AlertDialog.Builder saveAlertDlgBuilder = new AlertDialog.Builder(mContext);
                        saveAlertDlgBuilder.setMessage(mContext.getString(R.string.ask_for_save_k_image));
                        saveAlertDlgBuilder.setPositiveButton(mContext.getString(R.string.start_capture), (dialog2, which) -> {

                                startIrexCapture(userName,finalType);

                        });

                        AlertDialog saveAlertDlg = saveAlertDlgBuilder.create();
                        saveAlertDlg.show();
                        Window window = saveAlertDlg.getWindow();
                        if (window != null) {
                            window.setGravity(Gravity.BOTTOM);
                            window.setLayout(WindowManager.LayoutParams.MATCH_PARENT, WindowManager.LayoutParams.WRAP_CONTENT);
                        }
                        saveAlertDlg.getButton(DialogInterface.BUTTON_NEGATIVE).setAllCaps(false);
                        saveAlertDlg.getButton(DialogInterface.BUTTON_POSITIVE).setAllCaps(false);
                        saveAlertDlg.getButton(DialogInterface.BUTTON_NEUTRAL).setAllCaps(false);
                    }
                    else
                    {
                        startIrexCapture(userName,finalType);
                    }


                }
                else {
                    Toast.makeText(MainActivity.this, "User Name is empty", Toast.LENGTH_LONG).show();
                }

            });
            irexCaptureDialog.show();

        }
    }

    private void identify(){

        if (mCameraHelper!=null && StreamType.STREAM_TYPE_IRIS == mStreamType )
        {
            int type = mCameraHelper.LEFT_EYE;

            if(buttonRightEye.isChecked())
            {
                type = mCameraHelper.RIGHT_EYE;
            }
            else if(buttonTwoEyes.isChecked())
            {
                type = mCameraHelper.TWO_EYES;
            }

            if(mCameraHelper.isEnrolling())
            {
                mCameraHelper.antStopEnroll(false);
                mHintTv.setText(null);
            }

            if (mCameraHelper.isIdentify()) {

                mCameraHelper.antStopIdentify();
                onUpdateIdentifyButtonStatus();
                showEnrollProgress(false);

                if(mPowerSavingMode && mCameraHelper.isPreviewing())
                {
                    stopStream();
                }

                needIdentify = false;
                return;
            }

            if(!mCameraHelper.isPreviewing())
            {
                stopStream();

                needIdentify = true;
                mVerifyBtn.setEnabled(false);
                return;
            }

            int length_byte = 0;
            for (int i = 0; i < irises.size(); i++) {
                length_byte += irises.get(i).getTemp().length;
            }

            byte[] all_byte = new byte[length_byte];
            int countLength = 0;
            for (int i = 0; i < irises.size(); i++) {
                byte[] b = irises.get(i).getTemp();
                System.arraycopy(b, 0, all_byte, countLength, b.length);
                countLength += b.length;
            }

            showEnrollProgress(true);

             mCameraHelper.antIrisIdentify(type,new IdentifyCallback() {
                @Override
                public void onSuccess(int distance,String uid) {
                    runOnUiThread(() -> {
                        mSoundManager.play(SOUND_IDENTIFY_SUCCESS,true);
                        setHintText(getString(R.string.user) + ": " + uid + "        " + distance + "cm",Color.BLUE);
                    });

                }

                @Override
                public void onProcess(int distance,int opening) {
                    runOnUiThread(() -> {
                        //todo
                    });
                }

                @Override
                public void onError(String msg) {
                    String dstMsg;
                    if(mCameraHelper.ERROR_CAMERA_NOT_OPEN == msg)
                    {
                        dstMsg = getString(R.string.text_camera_not_open);setHintText(dstMsg,Color.RED);
                    }
                    else if(mCameraHelper.ERROR_IRIS_NOT_ENROLL  == msg)
                    {
                        dstMsg = getString(R.string.text_iris_not_enroll);setHintText(dstMsg,Color.RED);
                        showEnrollProgress(false);
                    }
                    else if(mCameraHelper.ERROR_ALGORITHM_NOT_ACTIVETED == msg)
                    {
                        dstMsg = getString(R.string.text_algorithm_not_activated);setHintText(dstMsg,Color.RED);
                    }
                    else if(mCameraHelper.ERROR_CLOSE_DISTANCE == msg)
                    {
                        mSoundManager.play(SOUND_MOVE_FURTHER,false);
                        dstMsg = getString(R.string.text_further);setHintText(dstMsg,Color.RED);
                        return;
                    }
                    else if(mCameraHelper.ERROR_FAR_DISTANCE == msg)
                    {
                        mSoundManager.play(SOUND_MOVE_CLOSER,false);
                        dstMsg = getString(R.string.text_closer);setHintText(dstMsg,Color.RED);
                        return;
                    }
                    else if(mCameraHelper.ERROR_EYE_NOT_WIDER_OPEN == msg)
                    {
                        mSoundManager.play(SOUND_OPEN_EYES_WIDER,false);
                        dstMsg = getString(R.string.open_eyes_wider);setHintText(dstMsg,Color.RED);
                        return;
                    }
                    else if(mCameraHelper.ERROR_EYE_EYE_GAZEANGLE == msg)
                    {
                        mSoundManager.play(SOUND_LOOK_CAMERA,false);
                        dstMsg = getString(R.string.look_camera);
                        setHintText(dstMsg,Color.RED);
                        return;
                    }
                    else
                    {
                        dstMsg = msg;
                    }

                    setHintText(getString(R.string.text_verify_fail) + " :" + dstMsg,Color.BLUE);

                    onUpdateIdentifyButtonStatus();
                }


            },all_byte,irises.size());

            onUpdateIdentifyButtonStatus();
        }
    }

    private  void onUpdateIdentifyButtonStatus()
    {
        if(mCameraHelper.isIdentify())
        {
            buttonRightEye.setEnabled(false);
            buttonLeftEye.setEnabled(false);
            buttonTwoEyes.setEnabled(false);
            mVerifyBtn.setText(R.string.stop_verify);
        }
        else
        {
            buttonRightEye.setEnabled(true);
            buttonLeftEye.setEnabled(true);
            buttonTwoEyes.setEnabled(true);
            mVerifyBtn.setText(R.string.text_verify);
        }
    }

    private void loadTemple() {

        irises.clear();
        
        byte[] IrisUsers = FileUtil.getFile(_templatePath + _irisUsersFileName);

        if(null != IrisUsers)
        {
            for (int i = 0; i < IrisUsers.length; i+=64)
            {
                byte[] userName = new byte[64];
                System.arraycopy(IrisUsers,i,userName,0,64);
                String sUserName = new String(userName);

                String finalUserName = sUserName.substring(0,sUserName.indexOf('\0'));

                byte[] templeBuffer = FileUtil.getFile(_templatePath + finalUserName + ".tmpl");
                if(null != templeBuffer && mCameraHelper.MAX_TEMPLATE_SIZE == templeBuffer.length )
                {
                    byte[] userName2 = new byte[64];
                    System.arraycopy(templeBuffer,0,userName2,0,64);
                    sUserName = new String(userName2);
                    finalUserName = sUserName.substring(0,sUserName.indexOf('\0'));

                   // Log.w(" read temple file,user name:" + finalUserName);

                    Iris iris = new Iris();
                    iris.setTemp(templeBuffer);
                    iris.setUser(finalUserName);
                    irises.add(iris );
                }

            }

            Log.d(" templates :" + irises.size());
        }
    }


    private void addTemple(byte [] data ,String userName)  {

        boolean finded = false;
        for (int i = 0; i < irises.size();i++)
        {
            if(userName.equals(irises.get(i).getUser()))
            {
                finded = true;
                break;
            }
        }

        if(!finded)
        {
            Iris iris = new Iris();
            iris.setTemp(data);
            iris.setUser(userName);
            irises.add(iris );
        }

      //  Log.i(TAG,"write  temple file :" + userName);
        if(FileUtil.saveFile(_templatePath,data, userName +".tmpl"))
        {
            byte[] userArray = new  byte[irises.size()*64];

            for (int i = 0; i < irises.size();i++)
            {
                byte[] byteUser = irises.get(i).getUser().getBytes();
                System.arraycopy(byteUser,0,userArray,64*i,byteUser.length);
            }

            FileUtil.saveFile(_templatePath,userArray,_irisUsersFileName);


        }

    }

    private void modifyTemple(int irisIndex,byte [] data){

        int leftEyeNum =  mCameraHelper.antIrisGetEyes(data,0);
        int rightEyeNum = mCameraHelper.antIrisGetEyes(data,1);

        Iris iris = irises.get(irisIndex);
        iris.setTemp(data);
        byte[] templeBuffer = iris.getTemp();

         leftEyeNum =  mCameraHelper.antIrisGetEyes(templeBuffer,0);
         rightEyeNum = mCameraHelper.antIrisGetEyes(templeBuffer,1);

        // Log.i(TAG,"modify  temple file :" + iris.getUser());

        FileUtil.saveFile(_templatePath,data, iris.getUser() +".tmpl");
    }


    public void onRadioButtonClicked( View view) {

        switch(view.getId()) {
            case R.id.left_eye:
                mCameraHelper.antSetEyeNum(0);
                break;
            case R.id.right_eye:
                mCameraHelper.antSetEyeNum(1);
                break;
        }
    }

}
