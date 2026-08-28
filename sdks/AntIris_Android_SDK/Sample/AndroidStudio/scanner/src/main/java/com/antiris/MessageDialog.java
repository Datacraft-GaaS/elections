package com.antiris;


import android.app.Dialog;
import android.content.Context;
import android.graphics.Point;
import android.os.Bundle;
import android.view.View;
import android.view.Window;
import android.view.WindowManager;
import android.widget.TextView;

import java.io.UnsupportedEncodingException;

public class MessageDialog extends Dialog implements View.OnClickListener {
    private String mMsg;
    private String mOKText;
    private String mCancelText;
    private PositiveListener positiveListener;

    public MessageDialog(String msg,Context context) {
        super(context, R.style.NumberInPasscode);
        setCancelable(false);
        setCanceledOnTouchOutside(true);

        mMsg = msg;
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.message_dialog);
        Window window = getWindow();
        WindowManager.LayoutParams layoutParams = window.getAttributes();
        layoutParams.dimAmount=0.75f;
        getWindow().setAttributes(layoutParams);
        getWindow().addFlags(WindowManager.LayoutParams.FLAG_DIM_BEHIND);

        WindowManager windowManager = getWindow().getWindowManager();
        Point point = new Point();
        windowManager.getDefaultDisplay().getRealSize(point);

        /*if(600 < point.x)
        {
            layoutParams.width = 600;
        }
        else
        {
            layoutParams.width = point.x;
        }

        window.setAttributes(layoutParams);*/

        TextView  title=findViewById(R.id.msg);
        title.setText(mMsg );

        TextView  cancel = findViewById(R.id.tv_cancel);
        cancel.setOnClickListener(this);
        cancel.setText(mCancelText);

        TextView  confirm= findViewById(R.id.tv_ok);
        confirm.setOnClickListener(this);
        confirm.setText(mOKText);

    }

    @Override
    public void onClick(View v) {

        switch (v.getId()){
            case R.id.tv_cancel:
                if (positiveListener!=null){
                    try {
                        positiveListener.setPositive(this,1);
                    } catch (UnsupportedEncodingException e) {
                        e.printStackTrace();
                    }
                }
                break;
            case R.id.tv_ok:
                if (positiveListener!=null){
                    try {
                        positiveListener.setPositive(this,0);
                    } catch (UnsupportedEncodingException e) {
                        e.printStackTrace();
                    }
                }
                break;
            default:break;
        }
    }

    public interface  PositiveListener{
        void setPositive(MessageDialog dialog, int msg) throws UnsupportedEncodingException;
    }

    public void setPositiveListener(String okText,String cancelText, PositiveListener l) {
        mOKText = okText;
        mCancelText = cancelText;
        positiveListener = l;
    }
}

