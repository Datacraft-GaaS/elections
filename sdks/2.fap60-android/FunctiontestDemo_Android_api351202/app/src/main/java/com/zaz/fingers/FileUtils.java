package com.zaz.fingers;


import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;


public class FileUtils
{
    public static byte[] readFile(String filePath)
    {
        File file = new File(filePath);
        long fileSize = file.length();

        if (file == null || !file.isFile())
        {
            return null;
        }

        InputStream is = null;
        try
        {
            is = new FileInputStream(file);
            byte[] buffer = new byte[(int) fileSize];
            int offset = 0;
            int numRead = 0;
            while (offset < buffer.length
                    && (numRead = is.read(buffer, offset, buffer.length - offset)) >= 0) {
                offset += numRead;
            }

            if (offset != buffer.length)
            {
                throw new IOException("Could not completely read file " + file.getName());
            }

            is.close();
            return buffer;

        }
        catch (IOException e)
        {
            throw new RuntimeException("IOException occurred. ", e);
        }
        finally
        {
            if (is != null)
            {
                try
                {
                    is.close();
                }
                catch (IOException e)
                {
                    throw new RuntimeException("IOException occurred. ", e);
                }
            }
        }
    }

    public static byte[] WriteHead(int nWidth, int nHeight) {

        int IMAGE_X = nWidth, IMAGE_Y = nHeight;
        byte[] head = new byte[1078];
        byte[] head1 = new byte[]{0x42, 0x4d,//file type
                //0x36,0x6c,0x01,0x00, //file size***
                0x0, 0x0, 0x0, 0x00, //file size***
                0x00, 0x00, //reserved
                0x00, 0x00,//reserved
                0x36, 0x4, 0x00, 0x00,//head byte***
                /***************************/
                //infoheader
                0x28, 0x00, 0x00, 0x00,//struct size

                //0x00,0x01,0x00,0x00,//map width***
                0x00, 0x00, 0x0, 0x00,//map width***
                //0x68,0x01,0x00,0x00,//map height***
                0x00, 0x00, 0x00, 0x00,//map height***

                0x01, 0x00,//must be 1
                0x08, 0x00,//color count***
                0x00, 0x00, 0x00, 0x00, //compression
                //0x00,0x68,0x01,0x00,//data size***
                0x00, 0x00, 0x00, 0x00,//data size***
                0x00, 0x00, 0x00, 0x00, //dpix
                0x00, 0x00, 0x00, 0x00, //dpiy
                0x00, 0x00, 0x00, 0x00,//color used
                0x00, 0x00, 0x00, 0x00,//color important

        };
        for (int z = 0; z < head1.length; z++) {
            head[z] = head1[z];
        }

        long num;
        num = IMAGE_X;
        head[18] = (byte) (num & 0xFF);
        num = num >> 8;
        head[19] = (byte) (num & 0xFF);
        num = num >> 8;
        head[20] = (byte) (num & 0xFF);
        num = num >> 8;
        head[21] = (byte) (num & 0xFF);


        num = IMAGE_Y;
        head[22] = (byte) (num & 0xFF);
        num = num >> 8;
        head[23] = (byte) (num & 0xFF);
        num = num >> 8;
        head[24] = (byte) (num & 0xFF);
        num = num >> 8;
        head[25] = (byte) (num & 0xFF);

        int i, j;

        j = 0;
        for (i = 54; i < 1078; i = i + 4) {
            head[i] = head[i + 1] = head[i + 2] = (byte) j;
            head[i + 3] = 0;
            j++;
        }


        return head;

    }

    public static boolean writeFile(String filePath, byte[] bytes) {
        if (bytes == null) {
            return false;
        }
        File outputFile = new File(filePath);
        FileOutputStream outputFileStream = null;
        try {
            makeDirs(filePath);
            outputFileStream = new FileOutputStream(outputFile);
            outputFileStream.write(bytes);
            outputFileStream.close();
            return true;
        } catch (IOException e) {
            throw new RuntimeException("IOException occurred. ", e);
        } finally {
            if (outputFileStream != null) {
                try {
                    outputFileStream.close();
                } catch (IOException e) {
                    throw new RuntimeException("IOException occurred. ", e);
                }
            }
        }
    }

    private byte[] MirrorVertically(byte[] picByte,int Width, int Height) {
        for (int y = 0; y < Height / 2; y++) {
            int swapY = Height - y - 1;
            for (int x = 0; x < Width; x++) {
                int index = y * Width + x;
                int swapIndex = swapY * Width + x;
                byte temp = picByte[index];
                picByte[index] = picByte[swapIndex];
                picByte[swapIndex] = temp;
            }
        }
        return picByte;
    }



    public static String getFolderName(String filePath) {

        if (isEmpty(filePath)) {
            return filePath;
        }

        int filePosi = filePath.lastIndexOf(File.separator);
        return (filePosi == -1) ? "" : filePath.substring(0, filePosi);
    }


    public static boolean makeDirs(String filePath)
    {
        String folderName = getFolderName(filePath);

        if (isEmpty(folderName))
        {
            return false;
        }

        File folder = new File(folderName);

        return (folder.exists() && folder.isDirectory()) ? true : folder.mkdirs();
    }

    public static boolean isEmpty(CharSequence str) {
        return (str == null || str.length() == 0);
    }
}
