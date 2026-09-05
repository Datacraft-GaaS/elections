package com.zaz.fingers;

import androidx.appcompat.app.AlertDialog;
import androidx.appcompat.app.AppCompatActivity;

import android.app.ActionBar;
import android.content.Context;
import android.content.DialogInterface;
import android.content.Intent;
import android.graphics.Bitmap;
import android.graphics.BitmapFactory;
import android.net.Uri;
import android.os.Bundle;
import android.os.Environment;
import android.provider.MediaStore;
import android.view.View;
import android.view.WindowManager;
import android.widget.LinearLayout;

import java.io.File;
import java.io.FileNotFoundException;
import java.io.FileOutputStream;
import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.channels.FileChannel;

public class showPhoto extends AppCompatActivity {
    private ZoomImageView zoomImageView;
    private LinearLayout linearLayout;
    private Bitmap bitmap;
    private String path;
    private byte[] Fimage;


    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_show_photo);
        getWindow().setFlags(WindowManager.LayoutParams.FLAG_FULLSCREEN, WindowManager.LayoutParams.FLAG_FULLSCREEN);//隐藏状态栏
        linearLayout=findViewById(R.id.root);
        path = getIntent().getStringExtra("photoPath");
        try {
            switch (path) {
                case "imagesplit":
                    ShowFingerImage(MainActivity.msplitfinger, 1600, 1500);
                    Fimage = MainActivity.msplitfinger;
                    break;
                case "roll":
                    ShowFingerImage(MainActivity.mfinger, 800, 750);
                    Fimage = MainActivity.mfinger;
                    break;
                case "df":
                    ShowFingerImage(MainActivity.mdfinger, 300, 400);
                    Fimage = MainActivity.mdfinger;
                    break;

            }
        } catch (Exception e) {

        }
        zoomImageView = (ZoomImageView) findViewById(R.id.zoom_image_view);
        zoomImageView.setImagePathBitmap(bitmap, 1.0f);
        zoomImageView.setResourceBitmap(this, bitmap);



    }
    private void ShowFingerImage(byte[] fingerImg, int w, int h) {


        byte[] bmpBytes = create8bitGrayscaleBMP(fingerImg, w, h);

        Bitmap bmp = BitmapFactory.decodeByteArray(bmpBytes, 0, bmpBytes.length);

            bitmap = bmp;

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
    void ShowFingerImage1(byte[] imageBuf, int w, int h) {
        Bitmap result = Bitmap.createBitmap(w, h, Bitmap.Config.RGB_565);

        int alpha = 0xFF << 24;
        int[] pixels = new int[w * h];    //通过位图的大小创建像素点数组

        for (int i = 0; i < h; i++) {
            for (int j = 0; j < w; j++) {
                int grey = imageBuf[w * i + j] & 0XFF;

                grey = alpha | (grey << 16) | (grey << 8) | grey;
                pixels[w * i + j] = grey;
            }
        }

        result.setPixels(pixels, 0, w, 0, 0, w, h);
        bitmap = result;

    }

    /**
     * 把数据流写入文件
     *
     * @param path
     * @param bytes
     */
    private void writeFile(String path, byte[] bytes) {
        try {
            FileOutputStream out = new FileOutputStream(path);//指定写到哪个路径中
            FileChannel fileChannel = out.getChannel();
            fileChannel.write(ByteBuffer.wrap(bytes)); //将字节流写入文件中
            fileChannel.force(true);//强制刷新
            fileChannel.close();
        } catch (FileNotFoundException e) {
            e.printStackTrace();
        } catch (IOException e) {
            e.printStackTrace();
        }
    }




}