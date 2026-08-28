package com.antiris;

import android.annotation.SuppressLint;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.graphics.Bitmap;
import android.hardware.usb.UsbDevice;
import android.os.Build;
import android.os.Bundle;
import android.os.Environment;
import android.text.TextUtils;
import android.view.View;
import android.view.Window;
import android.view.WindowManager;
import android.widget.AdapterView;
import android.widget.ArrayAdapter;
import android.widget.Button;
import android.widget.CheckBox;
import android.widget.ImageView;
import android.widget.RadioButton;
import android.widget.RadioGroup;
import android.widget.SeekBar;
import android.widget.Spinner;
import android.widget.TextView;
import android.widget.Toast;

import androidx.annotation.NonNull;
import androidx.annotation.Nullable;
import androidx.annotation.RequiresApi;
import androidx.appcompat.app.AppCompatActivity;

import com.antiris.AntUSBCamera.AuthCallback;
import com.antiris.AntUSBCamera.DeviceConnectCallback;
import com.antiris.AntUSBCamera.IrexCaptureCallback;
import com.antiris.AntUSBCamera.StreamSpeedLevel;
import com.antiris.AntUSBCamera.USBControl;
import com.antiris.AntUSBCamera.iso.IrisIso;
import com.antiris.AntUSBCamera.utils.BitmapUtil;
import com.antiris.AntUSBCamera.utils.FileUtil;
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
import java.util.Date;

public class MainActivity extends AppCompatActivity  implements
        DeviceConnectCallback,View.OnClickListener,RadioGroup.OnCheckedChangeListener ,AuthCallback{
    private static final String TAG = "Antiris";
    public View mUSBCameraImageTexView;
    static private USBControl mCameraHelper;
    private CustomProgressCircle _enrollProgress;
    private Button mEnrollBtn;
    private  Button _btnAccept;
    private  Button _btnCancel;
    private  Button _btnCapture;
    private SeekBar _seekBarExposure;
    private SoundManager soundManager;

    private TextView _mHintTv;
    private CheckBox auth;
    private CheckBox _checkLed;
    private CheckBox mConnected;
    private  CheckBox _mirror;
    private  ImageView _k7ImageLeft;
    private  ImageView _k7ImageRight;
    private Draw draw;
    private Timer  mTimer = null;
    private   long _showHintTextTimeMillis = 0;
    private  int _progressExposure = 684;
    private  int mProgress = 0;

    private RadioButton buttonLeftEye;
    private RadioButton buttonRightEye;
    private RadioButton buttonK1;
    private RadioButton buttonK2;
    private RadioButton buttonK3;
    private RadioButton buttonK7;
    private  Spinner mSpinnerSpeedLevel;
    private  String mConfigFilePath;

    static private final int SOUND_ENROLL_SUCCESS = 1;
    static private final int SOUND_IDENTIFY_SUCCESS = 2;
    static private final int SOUND_MOVE_FURTHER = 3;
    static private final int SOUND_MOVE_CLOSER = 4;
    static private final int SOUND_OPEN_EYES_WIDER = 5;
    static private final int SOUND_BEEP = 6;
    static LogCat logcat;

    private boolean mRestartPreview = false;

    private  boolean mViewMirror;


    private  String  _userName;

    protected  void onCreate(@Nullable Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        logcat = new LogCat();
        logcat.start(this,"224.0.2.1",8100,false,"");

        requestWindowFeature(Window.FEATURE_NO_TITLE);
        getWindow().addFlags(WindowManager.LayoutParams.FLAG_FULLSCREEN);
        setContentView(setLayoutRes());

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

    public  void init()
    {
        initView();

        try {
            initData();
        } catch (NoSuchAlgorithmException e) {
            e.printStackTrace();
        } catch (IOException e) {
            e.printStackTrace();
        }

        initListener();
    }


    public int setLayoutRes(){
        return R.layout.activity_camera;
    }


    int mSpeedLevel = StreamSpeedLevel.HIGH;

    private  void saveConfig()
        {
            try {

                JSONObject jsonObject = new JSONObject();
                jsonObject.put("speed_level", mSpeedLevel);

                File file = new File(getExternalFilesDir(null).getAbsolutePath(), "config"); // 内部存储

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
            if (file.exists())
            {

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

                mSpeedLevel = jsonObject.getInt("speed_level");

                Log.e(Log.TAG,"mSpeedLevel " + mSpeedLevel);
            }
            else
            {
                Log.d("read config is not found");

                mSpeedLevel = StreamSpeedLevel.HIGH;
            }



        } catch (Exception e) {
            e.printStackTrace();
            return ;
        }
    }

    public void initView() {
        mUSBCameraImageTexView=findViewById(R.id.camera_view);
        mUSBCameraImageTexView.setScaleX(-1);
        _enrollProgress = findViewById(R.id.enroll_progress);

        draw=findViewById(R.id.oval);
        buttonLeftEye = findViewById(R.id.left_eye);
        buttonRightEye = findViewById(R.id.right_eye);
        buttonK1 = findViewById(R.id.k1);
        buttonK2 = findViewById(R.id.k2);
        buttonK3 = findViewById(R.id.k3);
        buttonK7 = findViewById(R.id.k7);
        _mirror = findViewById(R.id.check_mirror);
       // _checkLed = findViewById(R.id.check_led);
        mConnected=findViewById(R.id.ck_camera);
        mEnrollBtn = findViewById(R.id.enrollBtn);
        _btnCancel = findViewById(R.id.cancel);
        _mHintTv = findViewById(R.id.hintTv);
        //_btnAccept = findViewById(R.id.accept);
        //_btnCapture = findViewById(R.id.capture);
        _k7ImageLeft = findViewById(R.id.k7imageLeft);
        _k7ImageRight = findViewById(R.id.k7imageRight);
        _seekBarExposure = findViewById(R.id.seekbar_exposure);

        mSpinnerSpeedLevel = (Spinner) findViewById(R.id.spinner_speed_level);
        String[] speedLevels = new String[]{
                "High Speed","Full Speed","Low Speed"
        };

        ArrayAdapter<String> adapter = new ArrayAdapter<>(this, android.R.layout.simple_spinner_item, speedLevels);
        adapter.setDropDownViewResource(android.R.layout.simple_spinner_dropdown_item);
        mSpinnerSpeedLevel.setAdapter(adapter);

        readConfig();

        mSpinnerSpeedLevel.setSelection(StreamSpeedLevel.HIGH - mSpeedLevel);

        mSpinnerSpeedLevel.setOnItemSelectedListener(new AdapterView.OnItemSelectedListener() {
            @Override
            public void onItemSelected(AdapterView<?> parent, View view, int position, long id) {
                if(null != mCameraHelper )
                {
                    if(mCameraHelper.isPreviewing())
                    {
                        mRestartPreview = true;
                    }

                    mSpeedLevel = StreamSpeedLevel.HIGH - position;

                    mCameraHelper.setStreamSpeedLevel(mSpeedLevel);

                    saveConfig();
                }
            }
            @Override
            public void onNothingSelected(AdapterView<?> parent) {
            }
        });
    }


    public void initData() throws NoSuchAlgorithmException, IOException {

        mCameraHelper = USBControl.getInstance();

        mCameraHelper.init(this, mUSBCameraImageTexView,this,true);

        modifyLayout(true);

        _enrollProgress.setMax(100);
        _enrollProgress.setProgress(0,0);

        showEnrollProgress(false);

        soundManager=new SoundManager(this);
        soundManager.put(SOUND_ENROLL_SUCCESS,R.raw.enrsucc_en);
        soundManager.put(SOUND_IDENTIFY_SUCCESS,R.raw.beep);
        soundManager.put(SOUND_MOVE_FURTHER,R.raw.move_further);
        soundManager.put(SOUND_MOVE_CLOSER,R.raw.move_closer);
        soundManager.put(SOUND_OPEN_EYES_WIDER,R.raw.open_eyes_wider);
        soundManager.put(SOUND_BEEP,R.raw.beep);
    }

    public void initListener() {
        mEnrollBtn.setOnClickListener(this);
//        _btnAccept.setOnClickListener(this);
        _btnCancel.setOnClickListener(this);
       // _btnCapture.setOnClickListener(this);
       // _checkLed.setOnClickListener(this);
        _mirror.setOnClickListener(this);

        _seekBarExposure.setOnSeekBarChangeListener(new SeekBar.OnSeekBarChangeListener() {
            @RequiresApi(api = Build.VERSION_CODES.JELLY_BEAN)
            @Override
            public void onProgressChanged(SeekBar seekBar, int progress, boolean fromUser) {
                _progressExposure = progress;
            }

            @Override
            public void onStartTrackingTouch(SeekBar seekBar) {
                //开始滑动时，执行此函数
            }

            @Override
            public void onStopTrackingTouch(SeekBar seekBar) {
                //结束滑动时，执行此函数
                if (mCameraHelper != null && mCameraHelper.isCameraOpened()) {
                    mCameraHelper.antSetExposure(0,_progressExposure);
                    TextView tvExposure = findViewById(R.id.text_exposure);
                    String exposureText = getString(R.string.exposure);
                    exposureText += ":";
                    exposureText += String.valueOf(_progressExposure);
                    tvExposure.setText(exposureText);
                }
            }
        });

    }

    @Override
    protected void onStart(){
        super.onStart();

        mTimer = new Timer();
        mTimer.schedule(1,1000,1000, (TimerCallback) timerID -> {
            if(0 != _showHintTextTimeMillis && System.currentTimeMillis() - _showHintTextTimeMillis > 2000)
            {
                _mHintTv.setText(null);
            }
        });
    }

    @Override
    protected void onStop() {
        super.onStop();

        if(null != mTimer)
        {
            mTimer.cancel();
            mTimer = null;
        }
    }

    @Override
    protected void onDestroy() {
        super.onDestroy();

        if (mCameraHelper != null) {
            mCameraHelper.release();
        }
    }

    private  void setHintText(String msg)
    {
        _mHintTv.setText(msg);
        _showHintTextTimeMillis = System.currentTimeMillis() ;
    }

    private void showShortMsg(String msg){
        Toast.makeText(this, msg, Toast.LENGTH_SHORT).show();
    }


    @Override
    public void onAttachDev(UsbDevice device) {

            if (mCameraHelper != null) {
                runOnUiThread(() -> {
                    mConnected.setChecked(false);
                    //_checkLed.setChecked(false);
                    mConnected.setText("connecting");
                });
            }

    }

    @Override
    public void onDettachDev(UsbDevice device) {

            runOnUiThread(() -> {
                mConnected.setChecked(false);
                //_checkLed.setChecked(false);
                mConnected.setText("Camera disconnect");
            });

    }

    @SuppressLint("SetTextI18n")
    @Override
    public void onConnectDev(UsbDevice device, boolean isConnected) {
        if (!isConnected) {
            showShortMsg("fail to connect,please check resolution params");
        } else {
            runOnUiThread(() -> {
                mConnected.setChecked(true);
                //_checkLed.setChecked(true);
                mConnected.setText("Camera Connected");

                mViewMirror = _mirror.isChecked();
                mUSBCameraImageTexView.setScaleX( mViewMirror ?1 : -1);

                modifyLayout(mCameraHelper.isMonocular());
            });
        }
    }


    @Override
    public void onDisConnectDev(UsbDevice device) {

    }

    @Override
    public void onPreview(boolean previewing) {

        runOnUiThread(() -> {
            if(previewing)
            {
                _btnCancel.setText(R.string.stop_stream);
                _btnCancel.setEnabled(true);
            }
            else
            {
                _btnCancel.setText(R.string.start_stream);
                _btnCancel.setEnabled(true);

                if(mRestartPreview)
                {
                    stopPreview();

                }
            }
        });

    }

    @Override
    public void onClick(View v) {
        switch (v.getId()){

            case R.id.enrollBtn:
                try {
                    irexCapture();
                } catch (IOException e) {
                    e.printStackTrace();
                }
                break;
            case R.id.cancel:
                stopPreview();
                break  ;

            case R.id.check_mirror:
            {
                mUSBCameraImageTexView.setScaleX(mViewMirror ? -1:1);
                mViewMirror = !mViewMirror;
            }
                break  ;
        }
    }

    private void ledOn(boolean on) {

        if(null != mCameraHelper && mCameraHelper.isCameraOpened())
        {
            mCameraHelper.IRLED_OnOff(on);
        }
    }

    private void stopPreview() {

        mRestartPreview = false;

        _btnCancel.setEnabled(false);
        showEnrollProgress(false);
        if(null != mCameraHelper)
        {
            mCameraHelper.stopStream( );
        }

        //this.finish();
    }

    private  int _progress  = 0;


    private void capture() {

        String _extStorageDirectory = Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_DOWNLOADS).toString();
        String filePath = _extStorageDirectory +"/antiris/";
        SimpleDateFormat format = new SimpleDateFormat("yyyy-MM-dd-HHmmss");

        String timestamp =format.format(new Date());
        String outName =  "bg_"+timestamp +".raw";

        byte[] bgImage = mCameraHelper.getBackgroundBytes(true);
        if(null != bgImage)
        {
            //save bg raw data
            FileUtil.saveFile(filePath,bgImage,outName);

            String bgBmpOutName =  "bg_"+timestamp +".bmp";
            //save bg bitmap

            mCameraHelper.saveGrayToBmp(bgImage,filePath,bgBmpOutName,mCameraHelper.getPreviewWidth(),mCameraHelper.getPreviewHeight(),false);
        }

    }



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

    int _irexType = 7;

    private  void startIrexCapture(String userName,int eyeIndex)  {
        _progress = 0;

        if(buttonK1.isChecked())
        {
            _irexType = 1;
        }
        else if(buttonK2.isChecked())
        {
            _irexType = 2;
        }
        else if(buttonK3.isChecked())
        {
            _irexType = 3;
        }
        else if(buttonK7.isChecked())
        {
            _irexType = 7;
        }

        showEnrollProgress(true);

        mCameraHelper.antIrisIrexCapture(userName,eyeIndex,_irexType,false,100*1000,new IrexCaptureCallback(){
            @Override
            public void onSuccess(int eyeIndex,String userName, byte[] temp){

                _userName = userName;

                Bitmap bm  = BitmapUtil.convert8bitToBitmap(mCameraHelper.getK7Image(eyeIndex),mCameraHelper.getK7ImgWidth(eyeIndex),mCameraHelper.getK7ImgHeight(eyeIndex));
                if(0 == eyeIndex)
                {
                    _k7ImageLeft.setImageBitmap(bm);
                }
                else
                {
                    _k7ImageRight.setImageBitmap(bm);
                }

                onUpdateEnrollButtonStatus();

                showEnrollProgress(false);

                String _extStorageDirectory = Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_DOWNLOADS).toString();
                String filePath = _extStorageDirectory +"/antiris/";
                SimpleDateFormat format = new SimpleDateFormat("yyyy-MM-dd-HHmmss");
                String timestamp =format.format(new Date());

                String eyeType = (0 == eyeIndex ? "left":"right");

                final  int k7ImageWidth = mCameraHelper.getK7ImgWidth(eyeIndex);
                final int k7ImageHeight = mCameraHelper.getK7ImgHeight(eyeIndex);
                final int k7Quality = mCameraHelper.getk7Quality(eyeIndex);

                byte[] k7Image = mCameraHelper.getK7Image(eyeIndex);
                if(null != k7Image)
                {
                    String k7OutName =  _userName + "_k" + _irexType +"_" +timestamp  +"_"+ k7ImageWidth +"x"+ k7ImageHeight +"_"+ eyeType + ".raw";
                    //save k7 raw data
                    FileUtil.saveFile(filePath,k7Image,k7OutName);

                    String k7PngOutName = _userName +  "_k" + _irexType +"_" +timestamp  +"_"+ eyeType +".png";
                    //save png file
                    mCameraHelper.saveGrayToPng(k7Image,filePath,k7PngOutName,k7ImageWidth,k7ImageHeight);

                    String k7Jp2OutName = userName +"_"+     eyeType  +"_"+  timestamp +".jp2";

                    /*mCameraHelper.saveGrayToJp2(k7Image,
                            filePath ,
                            k7Jp2OutName,
                            k7ImageWidth,
                            k7ImageHeight,
                            3500);*/
                    byte[] jp2Data = new byte[k7ImageWidth * k7ImageHeight];
                    int[] jp2Size = new int[1];

                    mCameraHelper.grayToJp2(k7Image,
                            k7ImageWidth,
                            k7ImageHeight,
                            jp2Data,
                            jp2Size,
                            _irexType,
                            mCameraHelper.ANT_COMPRESSION_MODE_LOSSY);
                    if ( jp2Size[0] > 0)
                    {
                        byte[] finalJp2Data = new byte[jp2Size[0]];

                        System.arraycopy(jp2Data,0,finalJp2Data,0,jp2Size[0]);

                        FileUtil.saveFile(filePath,finalJp2Data,k7Jp2OutName);

                                byte[] finalData = IrisIso.generate(0,finalJp2Data,
                                        k7ImageWidth,
                                        k7ImageHeight,
                                        mCameraHelper.getCameraSerialNumber().getBytes(StandardCharsets.UTF_8),
                                        mCameraHelper.getk7Quality(eyeIndex),
                                        _irexType,
                                        mCameraHelper.ANT_IMAGE_FORMAT_JPEG2000,
                                        mCameraHelper.ANT_COMPRESSION_MODE_LOSSY,
                                        8);
                                String isoOutName = userName +"_"+     eyeType  +"_"+ k7Quality +  "_" + timestamp + ".iso";

                                //save iso file
                                FileUtil.saveFile(filePath,finalData,isoOutName);



                    }

                }



            }


            @Override
            public void onError( String msg) {
                //Log.i(TAG, "enroll error: ");
                if(!msg.isEmpty())
                {
                    String hintText = "";
                    if(mCameraHelper.ERROR_CLOSE_DISTANCE == msg)
                    {
                        //soundManager.play(SOUND_MOVE_FURTHER,false);
                        hintText += "\n" + getString(R.string.text_further);
                        return;
                    }
                    else if(mCameraHelper.ERROR_FAR_DISTANCE == msg)
                    {
                        //soundManager.play(SOUND_MOVE_CLOSER,false);
                        hintText += "\n" + getString(R.string.text_closer);
                        return;
                    }
                    else if(mCameraHelper.ERROR_EYE_NOT_WIDER_OPEN == msg)
                    {
                        //soundManager.play(SOUND_OPEN_EYES_WIDER,false);
                        hintText += "\n" + getString(R.string.open_eyes_wider);
                        return;
                    }
                    else if(mCameraHelper.ERROR_EYE_EYE_GAZEANGLE == msg)
                    {
                        return;
                    }
                    else if(mCameraHelper.ERROR_KEEP_DISTANCE == msg)
                    {

                        return;
                    }
                    else {
                        setHintText(getString(R.string.text_enroll_fail) + ":" + msg);
                    }

                }
                else
                {
                    setHintText(msg);
                }

                onUpdateEnrollButtonStatus();

                showEnrollProgress(false);
            }

            @Override
            public void onProcess(int eyeNum,int progress, int distance,int opening) {

                if (0 < progress && 100 > progress)
                {
                    mProgress = progress;

                    if (_enrollProgress.getProgress(eyeNum) != progress ) {
                        _enrollProgress.setProgress(eyeNum,progress);
                    }

                }

                if(0 < distance)
                {
                    setHintText(getString(R.string.text_enroll_progress) + mProgress + "     " + distance + "cm");
                }

            }


        });

        onUpdateEnrollButtonStatus();
    }

    private void onUpdateEnrollButtonStatus()
    {
        if(mCameraHelper.isEnrolling())
        {
            mEnrollBtn.setText(R.string.stop_enroll);
        }
        else
        {
            mEnrollBtn.setText(R.string.text_enroll);
        }
    }

    private IrexCaptureDialog verifyDialog;

    private void irexCapture() throws IOException {

        if (mCameraHelper!=null && mCameraHelper.isCameraOpened()){
            if (mCameraHelper.isIrexCapturing()) {
               // ToastUtil.ToastNormal(getString(R.string.text_enrolling));
                mCameraHelper.antStopIrexCapture(true);
                _mHintTv.setText(null);
                onUpdateEnrollButtonStatus();
                _enrollProgress.setProgress(2,0);
                showEnrollProgress(false);
                return;
            }

            int eyeIndex = (buttonLeftEye.isChecked()?0:1);

            verifyDialog=new IrexCaptureDialog(this);
            verifyDialog.setPositiveListener("Done", (dialog, msg) -> {
                if (!TextUtils.isEmpty(msg))
                {
                    dialog.dismiss();

                    startIrexCapture(msg,eyeIndex);
                }
                else {
                    Toast.makeText(MainActivity.this, "User Name is empty", Toast.LENGTH_LONG).show();
                }

            });
            verifyDialog.show();

        }
    }


    @Override
    public void onAuthListenerCallback(boolean isSuccess) {
        Log.i(TAG, "onAuthListenerCallback: "+isSuccess);
        //auth.setChecked(isSuccess);
    }

    @Override
    public void onCheckedChanged(RadioGroup group, int checkedId) {
    }

    private void modifyLayout(boolean isMonocular){
        if(!isMonocular)
        {
            buttonLeftEye.setVisibility(View.GONE);
            buttonRightEye.setVisibility(View.GONE);
        }
        else
        {
            buttonLeftEye.setVisibility(View.VISIBLE);
            buttonRightEye.setVisibility(View.VISIBLE);
        }
    }


}
