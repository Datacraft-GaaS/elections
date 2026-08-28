package com.antiris;

import android.app.Dialog;
import android.content.Context;
import android.graphics.Point;
import android.os.Bundle;
import android.view.View;
import android.view.Window;
import android.view.WindowManager;
import android.widget.EditText;
import android.widget.TextView;

import java.io.UnsupportedEncodingException;

public class InputDialog extends Dialog implements View.OnClickListener {
    private String mTitle;
    private CharSequence mPositiveText;
    private EditText officer;
    private PositiveListener positiveListener;

    public InputDialog(String title,Context context) {
        super(context, R.style.NumberInPasscode);
        setCancelable(false);
        setCanceledOnTouchOutside(true);

        mTitle = title;
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.input_dialog);
        Window window = getWindow();
        WindowManager.LayoutParams layoutParams = window.getAttributes();
        layoutParams.dimAmount=0.75f;
        getWindow().setAttributes(layoutParams);
        getWindow().addFlags(WindowManager.LayoutParams.FLAG_DIM_BEHIND);

        WindowManager windowManager = getWindow().getWindowManager();
        Point point = new Point();
        windowManager.getDefaultDisplay().getRealSize(point);

        if(600 < point.x)
        {
            layoutParams.width = 600;
        }
        else
        {
            layoutParams.width = point.x;
        }

       // layoutParams.height = 300;

       // layoutParams.gravity = Gravity.CENTER;
        window.setAttributes(layoutParams);

        TextView  title=findViewById(R.id.lottieAnimationView);
        title.setText(mTitle );

       // title.setText(R.string.iris_recogniton);

        officer=findViewById(R.id.et_id);
        TextView  confirm=findViewById(R.id.tv_ok);
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

    public interface  PositiveListener{
        void setPositive(InputDialog dialog, String msg) throws UnsupportedEncodingException;
    }

    public void setPositiveListener(CharSequence text, PositiveListener l) {
        mPositiveText = text;
        positiveListener = l;
    }


}

