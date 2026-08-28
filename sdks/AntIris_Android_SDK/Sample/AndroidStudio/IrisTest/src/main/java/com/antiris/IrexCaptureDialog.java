package com.antiris;

import android.app.Dialog;
import android.content.Context;
import android.os.Bundle;
import android.view.Gravity;
import android.view.View;
import android.view.Window;
import android.view.WindowManager;
import android.widget.EditText;
import android.widget.TextView;


import com.antiris.R;

import java.io.UnsupportedEncodingException;

public class IrexCaptureDialog extends Dialog implements View.OnClickListener {
    private TextView confirm;

    private CharSequence mPositiveText;
    private EditText officer;

    public IrexCaptureDialog(Context context) {
        super(context, R.style.NumberInPasscode);
        setCancelable(false);
        setCanceledOnTouchOutside(true);
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.dialog_message);
        Window window = getWindow();
        WindowManager.LayoutParams layoutParams = window.getAttributes();
        layoutParams.dimAmount=0.75f;
        getWindow().setAttributes(layoutParams);
        getWindow().addFlags(WindowManager.LayoutParams.FLAG_DIM_BEHIND);
        layoutParams.width = 700;
        layoutParams.height = 600;
        layoutParams.gravity = Gravity.CENTER;
        window.setAttributes(layoutParams);
        officer=findViewById(R.id.et_id);
        confirm=findViewById(R.id.tv_ok);
        confirm.setOnClickListener(this);
        confirm.setText(mPositiveText);

    }

    @Override
    public void onClick(View v) {
        switch (v.getId()){
            case R.id.tv_ok:
                if (positiveListener!=null){
                    try {
                        positiveListener.setPositive(this,officer.getText().toString());
                    } catch (UnsupportedEncodingException e) {
                        e.printStackTrace();
                    }
                }
                break;
        }
    }


    public PositiveListener positiveListener;

    public interface  PositiveListener{
        void setPositive(IrexCaptureDialog dialog, String msg) throws UnsupportedEncodingException;
    }


    public void setPositiveListener(CharSequence text, PositiveListener l) {
        mPositiveText = text;
        positiveListener = l;
    }


}

